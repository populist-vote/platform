use crate::{context::ApiContext, guard::OrganizationGuard, is_admin};

use super::{organization_politician_note::OrganizationPoliticianNoteResult, IssueTagResult};
use async_graphql::*;
use db::{Organization, OrganizationPoliticianNote, OrganizationRoleType};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationAssets {
    thumbnail_image_160: Option<String>,
    thumbnail_image_400: Option<String>,
    banner_image: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationAttributes {
    supported_languages: Option<Vec<String>>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OrganizationResult {
    id: ID,
    slug: String,
    name: String,
    description: Option<String>,
    #[graphql(deprecation = "Use `assets.thumbnailImage160` instead")]
    thumbnail_image_url: Option<String>,
    website_url: Option<String>,
    facebook_url: Option<String>,
    twitter_url: Option<String>,
    instagram_url: Option<String>,
    email: Option<String>,
    votesmart_sig_id: Option<i32>,
    headquarters_address_id: Option<ID>,
    headquarters_phone: Option<String>,
    tax_classification: Option<String>,
    assets: OrganizationAssets,
    attributes: OrganizationAttributes,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(visible = "is_admin")]
pub struct OrganizationMemberResult {
    id: ID,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    profile_picture_url: Option<String>,
    role: OrganizationRoleType,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(visible = "is_admin")]
pub struct PendingInviteResult {
    email: String,
    role: Option<OrganizationRoleType>,
    created_at: chrono::DateTime<chrono::Utc>,
    accepted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[ComplexObject]
impl OrganizationResult {
    async fn issue_tags(&self, ctx: &Context<'_>) -> FieldResult<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records =
            Organization::issue_tags(&db_pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(IssueTagResult::from).collect();
        Ok(results)
    }

    async fn politician_notes(
        &self,
        ctx: &Context<'_>,
        election_id: ID,
    ) -> Result<Vec<OrganizationPoliticianNoteResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            OrganizationPoliticianNote,
            r#"
            SELECT
                id,
                organization_id,
                politician_id,
                election_id,
                notes,
                issue_tag_ids,
                created_at,
                updated_at
            FROM
                organization_politician_notes
            WHERE
                election_id = $1 AND
                organization_id = $2
        "#,
            uuid::Uuid::parse_str(&election_id).unwrap(),
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(records
            .into_iter()
            .map(OrganizationPoliticianNoteResult::from)
            .collect())
    }

    #[graphql(
        guard = "OrganizationGuard::new(&self.id, &OrganizationRoleType::ReadOnly)",
        visible = "is_admin"
    )]
    async fn members(&self, ctx: &Context<'_>) -> FieldResult<Vec<OrganizationMemberResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            OrganizationMemberResult,
            r#"
            SELECT u.id, ou.role AS "role:OrganizationRoleType", u.email, up.first_name, up.last_name, up.profile_picture_url
            FROM organization_users ou
            JOIN populist_user u ON ou.user_id = u.id
            JOIN user_profile up ON u.id = up.user_id
            WHERE ou.organization_id = $1
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        Ok(records)
    }

    #[graphql(
        guard = "OrganizationGuard::new(&self.id, &OrganizationRoleType::ReadOnly)",
        visible = "is_admin"
    )]
    async fn pending_invites(&self, ctx: &Context<'_>) -> FieldResult<Vec<PendingInviteResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query!(
            r#"
            SELECT i.email, i.role AS "role:OrganizationRoleType", i.created_at, i.accepted_at, i.organization_id FROM invite_token i
            WHERE i.organization_id = $1
            AND i.accepted_at IS NULL
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        let results = records
            .into_iter()
            .map(|r| PendingInviteResult {
                email: r.email,
                role: r.role.map(OrganizationRoleType::from),
                created_at: r.created_at,
                accepted_at: r.accepted_at,
            })
            .collect();
        Ok(results)
    }
}

impl From<Organization> for OrganizationResult {
    fn from(o: Organization) -> Self {
        Self {
            id: ID::from(o.id),
            slug: o.slug,
            name: o.name,
            description: o.description,
            thumbnail_image_url: o.thumbnail_image_url,
            website_url: o.website_url,
            facebook_url: o.facebook_url,
            twitter_url: o.twitter_url,
            instagram_url: o.instagram_url,
            email: o.email,
            votesmart_sig_id: o.votesmart_sig_id,
            headquarters_address_id: o.headquarters_address_id.map(ID::from),
            headquarters_phone: o.headquarters_phone,
            tax_classification: o.tax_classification,
            assets: serde_json::from_value(o.assets).unwrap_or_default(),
            attributes: serde_json::from_value(o.attributes).unwrap_or_default(),
        }
    }
}
