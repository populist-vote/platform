use crate::{context::ApiContext, types::OfficeResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{
        enums::{PoliticalParty, RaceType, State},
        politician::Politician,
        race::Race,
    },
    DateTime, Office,
};

use super::PoliticianResult;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct RaceResult {
    id: ID,
    slug: String,
    title: String,
    office_id: ID,
    race_type: RaceType,
    party: Option<PoliticalParty>,
    state: Option<State>,
    description: Option<String>,
    ballotpedia_link: Option<String>,
    early_voting_begins_date: Option<chrono::NaiveDate>,
    winner_id: Option<ID>,
    official_website: Option<String>,
    election_id: Option<ID>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl RaceResult {
    async fn office(&self, ctx: &Context<'_>) -> Result<OfficeResult> {
        let cached_office = ctx
            .data::<ApiContext>()?
            .loaders
            .office_loader
            .load_one(uuid::Uuid::parse_str(&self.office_id.clone()).unwrap())
            .await?;

        if let Some(office) = cached_office {
            Ok(OfficeResult::from(office))
        } else {
            let db_pool = ctx.data::<ApiContext>()?.pool.clone();
            let record =
                Office::find_by_id(&db_pool, uuid::Uuid::parse_str(&self.office_id).unwrap())
                    .await?;
            Ok(record.into())
        }
    }

    async fn candidates(&self, ctx: &Context<'_>) -> Result<Vec<PoliticianResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let politician_records = sqlx::query_as!(
            Politician,
            r#"
                SELECT
                    id,
                    slug,
                    first_name,
                    middle_name,
                    last_name,
                    nickname,
                    preferred_name,
                    ballot_name,
                    description,
                    home_state AS "home_state:State",
                    office_id,
                    thumbnail_image_url,
                    website_url,
                    facebook_url,
                    twitter_url,
                    instagram_url,
                    party AS "party:PoliticalParty",
                    votesmart_candidate_id,
                    votesmart_candidate_bio,
                    votesmart_candidate_ratings,
                    legiscan_people_id,
                    crp_candidate_id, 
                    fec_candidate_id,
                    upcoming_race_id,
                    p.created_at,
                    p.updated_at
                FROM
                    politician p
                    JOIN race_candidates rc ON race_id = $1
                WHERE
                    p.id = rc.candidate_id
                    AND rc.is_running = TRUE
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        let results = politician_records
            .into_iter()
            .map(PoliticianResult::from)
            .collect();
        Ok(results)
    }

    async fn election_date(&self, ctx: &Context<'_>) -> Result<Option<chrono::NaiveDate>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query!(
            r#"
            SELECT election_date FROM election
            WHERE id = $1
            "#,
            uuid::Uuid::parse_str(self.election_id.clone().unwrap_or_default().as_str()).unwrap()
        )
        .fetch_optional(&db_pool)
        .await?;

        Ok(record.map(|r| r.election_date))
    }
}

impl From<Race> for RaceResult {
    fn from(r: Race) -> Self {
        Self {
            id: ID::from(r.id),
            slug: r.slug,
            title: r.title,
            office_id: ID::from(r.office_id),
            race_type: r.race_type,
            party: r.party,
            state: r.state,
            description: r.description,
            ballotpedia_link: r.ballotpedia_link,
            early_voting_begins_date: r.early_voting_begins_date,
            winner_id: r.winner_id.map(ID::from),
            official_website: r.official_website,
            election_id: r.election_id.map(ID::from),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
