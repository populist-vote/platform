use crate::context::ApiContext;

use super::{organization_politician_note::OrganizationPoliticianNoteResult, IssueTagResult};
use async_graphql::*;
use db::{Organization, OrganizationPoliticianNote};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationAssets {
    thumbnail_image_160: Option<String>,
    thumbnail_image_400: Option<String>,
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
        }
    }
}
