use crate::types::OfficeResult;
use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::{
    models::{
        enums::{PoliticalParty, State},
        politician::Politician,
        race::Race,
    },
    DateTime, Office,
};
use sqlx::{Pool, Postgres};

use super::PoliticianResult;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct RaceResult {
    id: ID,
    slug: String,
    title: String,
    office_position: String,
    office_id: ID,
    race_type: String,
    state: Option<State>,
    description: Option<String>,
    ballotpedia_link: Option<String>,
    early_voting_begins_date: Option<chrono::NaiveDate>,
    election_date: Option<chrono::NaiveDate>,
    official_website: Option<String>,
    election_id: Option<ID>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl RaceResult {
    async fn office(&self, ctx: &Context<'_>) -> FieldResult<OfficeResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let record = Office::find_by_id(
            db_pool,
            uuid::Uuid::parse_str(&self.office_id.as_str()).unwrap(),
        )
        .await?;
        Ok(OfficeResult::from(record))
    }

    async fn candidates(&self, ctx: &Context<'_>) -> FieldResult<Vec<PoliticianResult>> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = sqlx::query_as!(Politician, r#"
            SELECT id, slug, first_name, middle_name, last_name, nickname, preferred_name, ballot_name, description, home_state AS "home_state:State", office_id, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, office_party AS "office_party:PoliticalParty", votesmart_candidate_id, votesmart_candidate_bio, votesmart_candidate_ratings, legiscan_people_id, upcoming_race_id, created_at, updated_at FROM politician
            WHERE upcoming_race_id = $1
            "#,
            uuid::Uuid::parse_str(self.id.as_str()).unwrap()
        ).fetch_all(db_pool).await?;
        let results = records.into_iter().map(PoliticianResult::from).collect();
        Ok(results)
    }
}

impl From<Race> for RaceResult {
    fn from(r: Race) -> Self {
        Self {
            id: ID::from(r.id),
            slug: r.slug,
            title: r.title,
            office_position: r.office_position,
            office_id: ID::from(r.office_id),
            race_type: r.race_type,
            state: r.state,
            description: r.description,
            ballotpedia_link: r.ballotpedia_link,
            early_voting_begins_date: r.early_voting_begins_date,
            election_date: r.election_date,
            official_website: r.official_website,
            election_id: match r.election_id {
                Some(id) => Some(ID::from(id)),
                None => None,
            },
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
