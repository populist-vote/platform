use crate::{
    context::ApiContext,
    relay,
    types::{PoliticalParty, PoliticianResult},
};
use async_graphql::{Context, Object, Result, ID};
use db::{
    loaders::politician::{PoliticianId, PoliticianSlug},
    models::enums::State,
    Politician, PoliticianFilter,
};

#[derive(Default, Debug)]
pub struct PoliticianQuery;

#[allow(clippy::too_many_arguments)]
#[Object]
impl PoliticianQuery {
    async fn politician_by_id(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<PoliticianResult>> {
        let politician = ctx
            .data::<ApiContext>()?
            .loaders
            .politician_loader
            .load_one(PoliticianId(uuid::Uuid::parse_str(&id)?))
            .await?;

        Ok(politician.map(PoliticianResult::from))
    }

    async fn politician_by_slug(
        &self,
        ctx: &Context<'_>,
        slug: String,
    ) -> Result<Option<PoliticianResult>> {
        let politician = ctx
            .data::<ApiContext>()?
            .loaders
            .politician_loader
            .load_one(PoliticianSlug(slug.clone()))
            .await?;

        Ok(politician.map(PoliticianResult::from))
    }

    async fn politician_by_intake_token(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<Option<PoliticianResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let politician = Politician::find_by_intake_token(&db_pool, token).await;
        if let Ok(politician) = politician {
            Ok(Some(PoliticianResult::from(politician)))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::needless_collect)]
    async fn politicians(
        &self,
        ctx: &Context<'_>,
        filter: Option<PoliticianFilter>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<PoliticianResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Politician::filter(&db_pool, &filter.unwrap_or_default()).await?;
        let results: Vec<PoliticianResult> =
            records.into_iter().map(PoliticianResult::from).collect();

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    async fn political_parties(&self, ctx: &Context<'_>) -> Result<Vec<PoliticalParty>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let parties = sqlx::query_as!(
            PoliticalParty,
            "SELECT id, fec_code, name, description, notes FROM party"
        )
        .fetch_all(&db_pool)
        .await?;
        Ok(parties)
    }

    async fn politician_respondents_by_organization_id(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
    ) -> Result<Vec<PoliticianResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            Politician,
            r#"
                SELECT DISTINCT ON (p.id)
                        p.id,
                        p.slug,
                        p.first_name,
                        p.middle_name,
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
                FROM
                    politician p
                JOIN
                    question_submission qs ON p.id = qs.candidate_id
                JOIN
                    question q ON qs.question_id = q.id
                WHERE
                    q.organization_id = $1
            "#,
            uuid::Uuid::parse_str(&organization_id)?
        )
        .fetch_all(&db_pool)
        .await?;

        let results: Vec<PoliticianResult> =
            records.into_iter().map(PoliticianResult::from).collect();

        Ok(results)
    }
}
