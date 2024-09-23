use crate::{
    context::{ApiContext, DataLoaders},
    guard::{IntakeTokenGuard, StaffOnly},
    is_admin,
    types::{Error, PoliticianResult},
    upload_to_s3, File,
};
use async_graphql::{Error as GraphQLError, *};
use db::{
    loaders::politician::PoliticianSlug, models::enums::State, CreateOrConnectIssueTagInput,
    CreateOrConnectOrganizationInput, CreateOrConnectPoliticianInput, InsertPoliticianInput,
    IssueTag, Organization, OrganizationIdentifier, Politician, PoliticianIdentifier,
    UpdatePoliticianInput,
};
use sqlx::{Pool, Postgres};
use std::io::Read;

use std::str::FromStr;
#[derive(Default)]
pub struct PoliticianMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct DeletePoliticianResult {
    id: String,
}

// Create or connect issue tags with relation to new or updated politician
async fn handle_nested_issue_tags(
    db_pool: &Pool<Postgres>,
    politician_id: uuid::Uuid,
    issue_tags_input: CreateOrConnectIssueTagInput,
) -> Result<(), Error> {
    if issue_tags_input.create.is_some() {
        for input in issue_tags_input.create.unwrap() {
            let new_issue_tag = IssueTag::upsert(db_pool, &input).await?;
            Politician::connect_issue_tag(db_pool, politician_id, new_issue_tag.id).await?;
        }
    }
    if issue_tags_input.connect.is_some() {
        for issue_tag_id in issue_tags_input.connect.unwrap() {
            // figure out how to accept slugs and IDs here, that'd be great
            Politician::connect_issue_tag(
                db_pool,
                politician_id,
                uuid::Uuid::parse_str(&issue_tag_id)?,
            )
            .await?;
        }
    }
    Ok(())
}

async fn handle_nested_organization_endorsements(
    db_pool: &Pool<Postgres>,
    politician_id: uuid::Uuid,
    organizations_input: CreateOrConnectOrganizationInput,
) -> Result<(), Error> {
    if organizations_input.create.is_some() {
        for input in organizations_input.create.unwrap() {
            let new_organization = Organization::create(db_pool, &input).await?;
            Politician::connect_organization(
                db_pool,
                politician_id,
                OrganizationIdentifier::Uuid(new_organization.id),
            )
            .await?;
        }
    }
    if organizations_input.connect.is_some() {
        for organization_identifier in organizations_input.connect.unwrap() {
            match uuid::Uuid::from_str(organization_identifier.as_str()) {
                Ok(org_id) => {
                    Politician::connect_organization(
                        db_pool,
                        politician_id,
                        OrganizationIdentifier::Uuid(org_id),
                    )
                    .await?
                }
                _ => {
                    Politician::connect_organization(
                        db_pool,
                        politician_id,
                        OrganizationIdentifier::Slug(organization_identifier),
                    )
                    .await?
                }
            };
        }
    }

    Ok(())
}

async fn handle_nested_politician_endorsements(
    db_pool: &Pool<Postgres>,
    politician_id: uuid::Uuid,
    politicians_input: CreateOrConnectPoliticianInput,
) -> Result<(), Error> {
    if politicians_input.create.is_some() {
        for input in politicians_input.create.unwrap() {
            let new_politician = Politician::insert(db_pool, &input).await?;
            Politician::connect_politician(
                db_pool,
                politician_id,
                PoliticianIdentifier::Uuid(new_politician.id),
            )
            .await?;
        }
    }
    if politicians_input.connect.is_some() {
        for politician_identifier in politicians_input.connect.unwrap() {
            match uuid::Uuid::from_str(politician_identifier.as_str()) {
                Ok(pol_endorsement_id) => {
                    Politician::connect_politician(
                        db_pool,
                        politician_id,
                        PoliticianIdentifier::Uuid(pol_endorsement_id),
                    )
                    .await?
                }
                _ => {
                    Politician::connect_politician(
                        db_pool,
                        politician_id,
                        PoliticianIdentifier::Slug(politician_identifier),
                    )
                    .await?
                }
            };
        }
    }

    Ok(())
}

#[Object]
impl PoliticianMutation {
    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn insert_politician(
        &self,
        ctx: &Context<'_>,
        input: InsertPoliticianInput,
    ) -> Result<PoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Politician::insert(&db_pool, &input).await?;
        // be sure to handle None inputs from GraphQL
        if input.issue_tags.is_some() {
            handle_nested_issue_tags(&db_pool, new_record.id, input.issue_tags.unwrap()).await?;
        }

        if input.organization_endorsements.is_some() {
            handle_nested_organization_endorsements(
                &db_pool,
                new_record.id,
                input.organization_endorsements.unwrap(),
            )
            .await?;
        }

        if input.politician_endorsements.is_some() {
            handle_nested_politician_endorsements(
                &db_pool,
                new_record.id,
                input.politician_endorsements.unwrap(),
            )
            .await?;
        }

        Ok(PoliticianResult::from(new_record))
    }

    #[graphql(
        guard = "IntakeTokenGuard::new(&_intake_token, &_slug)",
        visible = "is_admin"
    )]
    async fn update_politician(
        &self,
        ctx: &Context<'_>,
        _intake_token: String, // Only used for the guard
        _slug: String,
        input: UpdatePoliticianInput,
    ) -> Result<PoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = Politician::update(&db_pool, &input).await?;
        // be sure to handle None inputs from GraphQL
        if input.issue_tags.is_some() {
            handle_nested_issue_tags(&db_pool, new_record.id, input.issue_tags.unwrap()).await?;
        }

        if input.organization_endorsements.is_some() {
            handle_nested_organization_endorsements(
                &db_pool,
                new_record.id,
                input.organization_endorsements.unwrap(),
            )
            .await?;
        }

        if input.politician_endorsements.is_some() {
            handle_nested_politician_endorsements(
                &db_pool,
                new_record.id,
                input.politician_endorsements.unwrap(),
            )
            .await?;
        }

        Ok(PoliticianResult::from(new_record))
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn remove_politician_office(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Politician::remove_office(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(true)
    }

    #[graphql(guard = "StaffOnly", visible = "is_admin")]
    async fn delete_politician(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeletePoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        Politician::delete(&db_pool, uuid::Uuid::parse_str(&id)?).await?;
        Ok(DeletePoliticianResult { id })
    }

    #[graphql(
        guard = "IntakeTokenGuard::new(&_intake_token, &slug)",
        visible = "is_admin"
    )]
    async fn upload_politician_picture(
        &self,
        ctx: &Context<'_>,
        _intake_token: String, // Only used for the guard
        slug: String,
        file: Upload,
    ) -> Result<String> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let upload = file.value(ctx).unwrap();
        let mut content = Vec::new();
        let filename = format!("{}-400", slug);
        let mimetype = upload.content_type.clone();

        upload.into_read().read_to_end(&mut content).unwrap();
        let file_info = File {
            id: ID::from(uuid::Uuid::new_v4()),
            filename,
            content,
            mimetype,
        };
        let url = upload_to_s3(file_info, "web-assets/politician-thumbnails".to_string()).await?;
        // Append last modified date because s3 path will remain the same and we want browser to cache, but refresh the image
        let url = format!("{}{}{}", url, "?lastmod=", chrono::Utc::now().timestamp());

        let result = sqlx::query_as!(
            Politician,
            r#"
            UPDATE politician SET assets = jsonb_set(jsonb_set(assets, '{thumbnailImage160}', $1::jsonb, true), '{thumbnailImage400}', $1::jsonb, true)
            WHERE slug = $2
            RETURNING id,
            slug,
            first_name,
            middle_name,
            last_name,
            suffix,
            preferred_name,
            full_name,
            biography,
            biography_source,
            home_state AS "home_state:State",
            date_of_birth,
            office_id,
            upcoming_race_id,
            thumbnail_image_url,
            assets,
            official_website_url,
            campaign_website_url,
            facebook_url,
            twitter_url,
            instagram_url,
            youtube_url,
            linkedin_url,
            tiktok_url,
            email,
            phone,
            party_id,
            votesmart_candidate_id,
            votesmart_candidate_bio,
            votesmart_candidate_ratings,
            legiscan_people_id,
            crp_candidate_id,
            fec_candidate_id,
            race_wins,
            race_losses,
            created_at,
            updated_at
        "#,
            serde_json::json!(url), // Convert url to JSON format
            slug
        )
        .fetch_one(&db_pool)
        .await;

        match result {
            Ok(politician) => {
                DataLoaders::new(db_pool)
                    .politician_loader
                    .feed_one(PoliticianSlug(slug), politician)
                    .await;
                Ok(url)
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Err(GraphQLError::from(err))
            }
        }
    }
}
