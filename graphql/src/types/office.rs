use crate::{context::ApiContext, types::PoliticianResult};
use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::{
    models::{
        enums::{PoliticalParty, PoliticalScope, State},
        office::Office,
        politician::Politician,
    },
    District, ElectionScope,
};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OfficeResult {
    id: ID,
    slug: String,
    title: String,
    office_type: Option<String>,
    district: Option<String>,
    district_type: Option<District>,
    political_scope: PoliticalScope,
    election_scope: ElectionScope,
    state: Option<State>,
    municipality: Option<String>,
}

#[ComplexObject]
impl OfficeResult {
    async fn incumbent(&self, ctx: &Context<'_>) -> FieldResult<Option<PoliticianResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
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
                WHERE office_id = $1
            "#,
            uuid::Uuid::parse_str(&self.id.as_str()).unwrap()
        )
        .fetch_optional(&db_pool)
        .await?;

        if let Some(record) = record {
            let politician_result = PoliticianResult::from(record);
            Ok(Some(politician_result))
        } else {
            Ok(None)
        }
    }
}

impl From<Office> for OfficeResult {
    fn from(o: Office) -> Self {
        Self {
            id: ID::from(o.id),
            slug: o.slug,
            title: o.title,
            office_type: o.office_type,
            district: o.district,
            district_type: o.district_type,
            political_scope: o.political_scope,
            election_scope: o.election_scope,
            state: o.state,
            municipality: o.municipality,
        }
    }
}
