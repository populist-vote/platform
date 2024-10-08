use crate::{
    models::enums::State, CreateOrConnectIssueTagInput, CreateOrConnectOrganizationInput, DateTime,
    IssueTag, Organization, OrganizationIdentifier,
};
use async_graphql::InputObject;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSON;
use slugify::slugify;
use sqlx::{postgres::PgPool, FromRow};

use super::enums::{Chambers, PoliticalScope};

#[derive(FromRow, Debug, Clone)]
pub struct Politician {
    pub id: uuid::Uuid,
    pub slug: String,
    pub ref_key: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub full_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub party_id: Option<uuid::Uuid>,
    pub date_of_birth: Option<NaiveDate>,
    pub office_id: Option<uuid::Uuid>,
    pub upcoming_race_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    pub assets: JSON,
    pub official_website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: JSON,
    pub votesmart_candidate_ratings: JSON,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub race_wins: Option<i32>,
    pub race_losses: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[serde_with::serde_as]
#[derive(InputObject, Debug, Default, Serialize, Deserialize)]
pub struct InsertPoliticianInput {
    pub slug: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub date_of_birth: Option<NaiveDate>,
    pub office_id: Option<uuid::Uuid>,
    pub upcoming_race_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    #[serde_as(as = "serde_with::json::JsonString")]
    pub assets: Option<JSON>,
    pub official_website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub party_id: Option<uuid::Uuid>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
    pub organization_endorsements: Option<CreateOrConnectOrganizationInput>,
    pub politician_endorsements: Option<CreateOrConnectPoliticianInput>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<JSON>,
    pub votesmart_candidate_ratings: Option<JSON>,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub race_wins: Option<i32>,
    pub race_losses: Option<i32>,
}

#[serde_with::serde_as]
#[derive(InputObject, Debug, Default, Serialize, Deserialize)]
pub struct UpdatePoliticianInput {
    pub id: uuid::Uuid,
    pub slug: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub date_of_birth: Option<NaiveDate>,
    pub office_id: Option<uuid::Uuid>,
    pub upcoming_race_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    #[serde_as(as = "serde_with::json::JsonString")]
    pub assets: Option<JSON>,
    pub official_website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub party_id: Option<uuid::Uuid>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
    pub organization_endorsements: Option<CreateOrConnectOrganizationInput>,
    pub politician_endorsements: Option<CreateOrConnectPoliticianInput>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<JSON>,
    pub votesmart_candidate_ratings: Option<JSON>,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub race_wins: Option<i32>,
    pub race_losses: Option<i32>,
}

#[serde_with::serde_as]
#[derive(InputObject, Debug, Default, Serialize, Deserialize)]
pub struct UpsertPoliticianInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub ref_key: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub full_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub date_of_birth: Option<NaiveDate>,
    pub office_id: Option<uuid::Uuid>,
    pub upcoming_race_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    #[serde_as(as = "serde_with::json::JsonString")]
    pub assets: Option<JSON>,
    pub official_website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub party_id: Option<uuid::Uuid>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
    pub organization_endorsements: Option<CreateOrConnectOrganizationInput>,
    pub politician_endorsements: Option<CreateOrConnectPoliticianInput>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<JSON>,
    pub votesmart_candidate_ratings: Option<JSON>,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub race_wins: Option<i32>,
    pub race_losses: Option<i32>,
}

pub enum PoliticianIdentifier {
    Uuid(uuid::Uuid),
    Slug(String),
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateOrConnectPoliticianInput {
    pub create: Option<Vec<InsertPoliticianInput>>,
    pub connect: Option<Vec<String>>, // Accept UUIDs or slugs
}

#[derive(InputObject, Default, Debug)]
pub struct PoliticianFilter {
    pub query: Option<String>,
    pub home_state: Option<State>,
    pub political_scope: Option<PoliticalScope>,
    pub chambers: Option<Chambers>,
}

impl Politician {
    pub async fn insert(
        db_pool: &PgPool,
        input: &InsertPoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        let slug = match &input.slug {
            Some(slug) => slug.to_owned(),
            None => slugify!(&format!(
                "{} {} {}",
                input
                    .preferred_name
                    .clone()
                    .unwrap_or_else(|| input.first_name.clone()),
                input.last_name.clone(),
                input.suffix.clone().unwrap_or_default()
            )),
        };

        let record = sqlx::query_as!(
            Politician,
            r#"
            INSERT INTO politician (slug, first_name, middle_name, last_name, suffix, preferred_name, biography, biography_source, home_state, date_of_birth, office_id, upcoming_race_id, thumbnail_image_url, assets, official_website_url, campaign_website_url, facebook_url, twitter_url, instagram_url, youtube_url, linkedin_url, tiktok_url, email, phone, party_id, votesmart_candidate_id, votesmart_candidate_bio, votesmart_candidate_ratings, legiscan_people_id, crp_candidate_id, fec_candidate_id, race_wins, race_losses)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33) 
            RETURNING
                id,
                slug,
                ref_key,
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
            slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.suffix,
            input.preferred_name,
            input.biography,
            input.biography_source,
            input.home_state as Option<State>,
            input.date_of_birth as Option<NaiveDate>,
            input.office_id,
            input.upcoming_race_id,
            input.thumbnail_image_url,
            input.assets,
            input.official_website_url,
            input.campaign_website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.youtube_url,
            input.linkedin_url,
            input.tiktok_url,
            input.email,
            input.phone,
            input.party_id,
            input.votesmart_candidate_id,
            input.votesmart_candidate_bio,
            input.votesmart_candidate_ratings,
            input.legiscan_people_id,
            input.crp_candidate_id,
            input.fec_candidate_id,
            input.race_wins,
            input.race_losses,
        ).fetch_one(db_pool).await?;

        Ok(record)
    }

    pub async fn upsert_from_source(
        db_pool: &PgPool,
        input: &UpsertPoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        input
            .ref_key
            .as_ref()
            .ok_or("ref_key is required")
            .map_err(|err| sqlx::Error::AnyDriverError(err.into()))?;

        input
            .slug
            .as_ref()
            .ok_or("slug is required")
            .map_err(|err| sqlx::Error::AnyDriverError(err.into()))?;

        let slug = sqlx::query_scalar!(
            r#"
            SELECT CASE
            WHEN EXISTS (
                SELECT slug
                FROM politician 
                WHERE ref_key = $1 
            ) THEN (
                SELECT slug 
                FROM politician 
                WHERE ref_key = $1 
            )
            WHEN EXISTS (
                SELECT 1 
                FROM politician 
                WHERE slug = $2 
            ) THEN
                $2 || '-' || COALESCE(
                    (
                        SELECT MAX(SUBSTRING(slug FROM LENGTH($2) + 2)::INT) 
                        FROM politician 
                        WHERE slug ~ ($2 || '-[0-9]+$')
                    ), 0
                ) + 1
            ELSE
                $2
            END
            "#,
            input.ref_key,
            input.slug,
        )
        .fetch_one(db_pool)
        .await?
        .ok_or_else(|| {
            sqlx::Error::AnyDriverError("Failure determining slug for Politician".into())
        })?;

        sqlx::query_as!(
            Politician,
            r#"
            INSERT INTO politician (ref_key, slug, first_name, middle_name, last_name, suffix, preferred_name, full_name, biography, biography_source, home_state, date_of_birth, office_id, upcoming_race_id, thumbnail_image_url, assets, official_website_url, campaign_website_url, facebook_url, twitter_url, instagram_url, youtube_url, linkedin_url, tiktok_url, email, phone, party_id, votesmart_candidate_id, votesmart_candidate_bio, votesmart_candidate_ratings, legiscan_people_id, crp_candidate_id, fec_candidate_id, race_wins, race_losses)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, COALESCE($16, '{}'::jsonb), $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, COALESCE($29, '{}'::jsonb), COALESCE($30, '{}'::jsonb), $31, $32, $33, $34, $35) 
            ON CONFLICT (ref_key) DO UPDATE SET
                slug = COALESCE($2, politician.slug),
                first_name = COALESCE($3, politician.first_name),
                middle_name = COALESCE($4, politician.middle_name),
                last_name = COALESCE($5, politician.last_name),
                suffix = COALESCE($6, politician.suffix),
                preferred_name = COALESCE($7, politician.preferred_name),
                full_name = COALESCE($8, politician.full_name),
                biography = COALESCE($9, politician.biography),
                biography_source = COALESCE($10, politician.biography_source),
                home_state = COALESCE($11, politician.home_state),
                date_of_birth = COALESCE($12, politician.date_of_birth),
                office_id = COALESCE($13, politician.office_id),
                upcoming_race_id = COALESCE($14, politician.upcoming_race_id),
                thumbnail_image_url = COALESCE($15, politician.thumbnail_image_url),
                assets = COALESCE($16, politician.assets),
                official_website_url = COALESCE($17, politician.official_website_url),
                campaign_website_url = COALESCE($18, politician.campaign_website_url),
                facebook_url = COALESCE($19, politician.facebook_url),
                twitter_url = COALESCE($20, politician.twitter_url),
                instagram_url = COALESCE($21, politician.instagram_url),
                youtube_url = COALESCE($22, politician.youtube_url),
                linkedin_url = COALESCE($23, politician.linkedin_url),
                tiktok_url = COALESCE($24, politician.tiktok_url),
                email = COALESCE($25, politician.email),
                phone = COALESCE($26, politician.phone),
                party_id = COALESCE($27, politician.party_id),
                votesmart_candidate_id = COALESCE($28, politician.votesmart_candidate_id),
                votesmart_candidate_bio = COALESCE($29, politician.votesmart_candidate_bio),
                votesmart_candidate_ratings = COALESCE($30, politician.votesmart_candidate_ratings),
                legiscan_people_id = COALESCE($31, politician.legiscan_people_id),
                crp_candidate_id = COALESCE($32, politician.crp_candidate_id),
                fec_candidate_id = COALESCE($33, politician.fec_candidate_id),
                race_wins = COALESCE($34, politician.race_wins),
                race_losses = COALESCE($35, politician.race_losses)
            RETURNING
                id,
                slug,
                ref_key,
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
            input.ref_key,
            &slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.suffix,
            input.preferred_name,
            input.full_name,
            input.biography,
            input.biography_source,
            input.home_state as Option<State>,
            input.date_of_birth as Option<NaiveDate>,
            input.office_id,
            input.upcoming_race_id,
            input.thumbnail_image_url,
            input.assets,
            input.official_website_url,
            input.campaign_website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.youtube_url,
            input.linkedin_url,
            input.tiktok_url,
            input.email,
            input.phone,
            input.party_id,
            input.votesmart_candidate_id,
            input.votesmart_candidate_bio,
            input.votesmart_candidate_ratings,
            input.legiscan_people_id,
            input.crp_candidate_id,
            input.fec_candidate_id,
            input.race_wins,
            input.race_losses,
        ).fetch_one(db_pool).await
    }

    pub async fn update(
        db_pool: &PgPool,
        input: &UpdatePoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
            UPDATE politician
            SET
                slug = COALESCE($2, politician.slug),
                first_name = COALESCE($3, politician.first_name),
                middle_name = COALESCE($4, politician.middle_name),
                last_name = COALESCE($5, politician.last_name),
                suffix = COALESCE($6, politician.suffix),
                preferred_name = COALESCE($7, politician.preferred_name),
                biography = COALESCE($8, politician.biography),
                biography_source = COALESCE($9, politician.biography_source),
                home_state = COALESCE($10, politician.home_state),
                date_of_birth = COALESCE($11, politician.date_of_birth),
                office_id = COALESCE($12, politician.office_id),
                upcoming_race_id = COALESCE($13, politician.upcoming_race_id),
                thumbnail_image_url = COALESCE($14, politician.thumbnail_image_url),
                assets = COALESCE($15, politician.assets),
                official_website_url = COALESCE($16, politician.official_website_url),
                campaign_website_url = COALESCE($17, politician.campaign_website_url),
                facebook_url = COALESCE($18, politician.facebook_url),
                twitter_url = COALESCE($19, politician.twitter_url),
                instagram_url = COALESCE($20, politician.instagram_url),
                youtube_url = COALESCE($21, politician.youtube_url),
                linkedin_url = COALESCE($22, politician.linkedin_url),
                tiktok_url = COALESCE($23, politician.tiktok_url),
                email = COALESCE($24, politician.email),
                phone = COALESCE($25, politician.phone),
                party_id = COALESCE($26, politician.party_id),
                votesmart_candidate_id = COALESCE($27, politician.votesmart_candidate_id),
                votesmart_candidate_bio = COALESCE($28, politician.votesmart_candidate_bio),
                votesmart_candidate_ratings = COALESCE($29, politician.votesmart_candidate_ratings),
                legiscan_people_id = COALESCE($30, politician.legiscan_people_id),
                crp_candidate_id = COALESCE($31, politician.crp_candidate_id),
                fec_candidate_id = COALESCE($32, politician.fec_candidate_id),
                race_wins = COALESCE($33, politician.race_wins),
                race_losses = COALESCE($34, politician.race_losses)
            WHERE id = $1
            RETURNING
                id,
                slug,
                ref_key,
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
            input.id,
            input.slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.suffix,
            input.preferred_name,
            input.biography,
            input.biography_source,
            input.home_state as Option<State>,
            input.date_of_birth as Option<NaiveDate>,
            input.office_id,
            input.upcoming_race_id,
            input.thumbnail_image_url,
            input.assets,
            input.official_website_url,
            input.campaign_website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.youtube_url,
            input.linkedin_url,
            input.tiktok_url,
            input.email,
            input.phone,
            input.party_id,
            input.votesmart_candidate_id,
            input.votesmart_candidate_bio,
            input.votesmart_candidate_ratings,
            input.legiscan_people_id,
            input.crp_candidate_id,
            input.fec_candidate_id,
            input.race_wins,
            input.race_losses,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn remove_office(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE politician SET office_id = NULL WHERE id = $1",
            politician_id
        )
        .execute(db_pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM politician WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Politician,
            r#"SELECT id,
                        slug,
                        ref_key,
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
                        updated_at FROM politician"#,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
                        slug,
                        ref_key,
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
                        updated_at FROM politician
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(record)
    }

    pub async fn find_by_slug(db_pool: &PgPool, slug: String) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
                        slug,
                        ref_key,
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
                        updated_at FROM politician
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_intake_token(
        db_pool: &PgPool,
        token: String,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
                        slug,
                        ref_key,
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
                        updated_at FROM politician
                WHERE intake_token = $1
            "#,
            token
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn filter(
        db_pool: &PgPool,
        filter: &PoliticianFilter,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let search_query = filter.query.to_owned().unwrap_or_default();
        let records = sqlx::query_as!(
            Politician,
            r#"
                SELECT  p.id,
                        p.slug,
                        p.ref_key,
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
                        p.created_at,
                        p.updated_at 
                FROM politician p
                LEFT JOIN office o ON office_id = o.id,
                to_tsvector(
                    first_name || ' ' || last_name || ' ' || COALESCE(preferred_name, '') || COALESCE(o.title, '')
                 ) document,
                websearch_to_tsquery($1::text) query,
                NULLIF(ts_rank(to_tsvector(first_name), query), 0) rank_first_name,
                NULLIF(ts_rank(to_tsvector(last_name), query), 0) rank_last_name,
                NULLIF(ts_rank(to_tsvector(preferred_name), query), 0) rank_preferred_name,
                NULLIF(ts_rank(to_tsvector(o.title), query), 0) rank_office_title
                WHERE (($1::text = '') IS NOT FALSE OR query @@ document)
                AND ($2::state IS NULL OR home_state = $2)
                AND ($3::political_scope IS NULL OR political_scope = $3)
                AND ($4::text IS NULL OR $4 = 'All' OR (
                    ($4 = 'Senate' AND o.title ILIKE '%Senator') OR
                    ($4 = 'House' AND o.title ILIKE '%Representative')
                ))
                GROUP BY (
                    p.id,
                    rank_last_name,
                    rank_preferred_name,
                    rank_first_name,
                    rank_office_title,
                    o.title
                )
                ORDER BY last_name ASC
                LIMIT 50
            "#,
            search_query,
            filter.home_state as Option<State>,
            filter.political_scope as Option<PoliticalScope>,
            filter.chambers.map(|c| c.to_string()) as Option<String>
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn organization_endorsements(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<Organization>, sqlx::Error> {
        let records = sqlx::query_as!(Organization,
            r#"
                SELECT o.id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, votesmart_sig_id, headquarters_address_id, headquarters_phone, tax_classification, o.politician_id, o.assets, o.attributes, o.created_at, o.updated_at  FROM organization o
                JOIN politician_organization_endorsements
                ON politician_organization_endorsements.organization_id = o.id
                WHERE politician_organization_endorsements.politician_id = $1
            "#, 
        politician_id).fetch_all(db_pool).await?;

        Ok(records)
    }

    pub async fn politician_endorsements(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<Politician>, sqlx::Error> {
        let records = sqlx::query_as!(
            Politician,
            r#"
                SELECT p.id,
                        slug,
                        ref_key,
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
                        p.created_at,
                        p.updated_at FROM politician p
                JOIN politician_politician_endorsements
                ON politician_politician_endorsements.politician_endorsement_id = p.id
                WHERE politician_politician_endorsements.politician_id = $1
            "#,
            politician_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn connect_organization(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
        organization_identifier: OrganizationIdentifier,
    ) -> Result<(), sqlx::Error> {
        match organization_identifier {
            OrganizationIdentifier::Uuid(organization_id) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_organization_endorsements (politician_id, organization_id)
                        VALUES ($1, $2)
                    "#,
                    politician_id,
                    organization_id
                )
                .execute(db_pool)
                .await?;
            }
            OrganizationIdentifier::Slug(organization_slug) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_organization_endorsements (politician_id, organization_id)
                        VALUES ($1, (SELECT id FROM organization WHERE slug = $2))
                    "#,
                    politician_id,
                    organization_slug
                )
                .execute(db_pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn connect_politician(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
        politician_identifier: PoliticianIdentifier,
    ) -> Result<(), sqlx::Error> {
        match politician_identifier {
            PoliticianIdentifier::Uuid(politician_endorsement_id) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_politician_endorsements (politician_id, politician_endorsement_id)
                        VALUES ($1, $2)
                    "#,
                    politician_id,
                    politician_endorsement_id
                )
                .execute(db_pool)
                .await?;
            }
            PoliticianIdentifier::Slug(politician_endorsement_slug) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_politician_endorsements (politician_id, politician_endorsement_id)
                        VALUES ($1, (SELECT id FROM politician WHERE slug = $2))
                    "#,
                    politician_id,
                    politician_endorsement_slug
                )
                .execute(db_pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn connect_issue_tag(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
        issue_tag_id: uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Politician,
            r#"
                INSERT INTO politician_issue_tags (politician_id, issue_tag_id) 
                VALUES ($1, $2)
            "#,
            politician_id,
            issue_tag_id
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN politician_issue_tags
                ON politician_issue_tags.issue_tag_id = it.id
                WHERE politician_issue_tags.politician_id = $1
            "#,
            politician_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
