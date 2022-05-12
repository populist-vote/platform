use crate::{context::ApiContext, types::OfficeResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use chrono::NaiveDate;
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

#[derive(SimpleObject, Debug, Clone)]
pub struct CandidateResult {
    politician: PoliticianResult,
    is_running: bool,
    date_announced: Option<NaiveDate>,
    date_qualified: Option<NaiveDate>,
    date_dropped: Option<NaiveDate>,
    reason_dropped: Option<String>,
    qualification_method: Option<String>,
    qualification_info: Option<String>,
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

    async fn candidates(&self, ctx: &Context<'_>) -> Result<Vec<CandidateResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let candidate_records = sqlx::query!(
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
                    upcoming_race_id,
                    p.created_at,
                    p.updated_at,
                    is_running,
                    date_announced,
                    date_qualified,
                    date_dropped,
                    reason_dropped,
                    qualification_method,
                    qualification_info
                FROM
                    politician p
                    JOIN race_candidates rc ON race_id = $1
                    WHERE p.id = rc.candidate_id
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        let candidate_results = candidate_records
            .into_iter()
            .map(|candidate_record| -> CandidateResult {
                let politician_record = Politician {
                    id: candidate_record.id,
                    slug: candidate_record.slug,
                    first_name: candidate_record.first_name,
                    middle_name: candidate_record.middle_name,
                    last_name: candidate_record.last_name,
                    nickname: candidate_record.nickname,
                    preferred_name: candidate_record.preferred_name,
                    ballot_name: candidate_record.ballot_name,
                    description: candidate_record.description,
                    home_state: candidate_record.home_state,
                    office_id: candidate_record.office_id,
                    thumbnail_image_url: candidate_record.thumbnail_image_url,
                    website_url: candidate_record.website_url,
                    twitter_url: candidate_record.twitter_url,
                    facebook_url: candidate_record.facebook_url,
                    instagram_url: candidate_record.instagram_url,
                    party: candidate_record.party,
                    legiscan_people_id: candidate_record.legiscan_people_id,
                    votesmart_candidate_id: candidate_record.votesmart_candidate_id,
                    votesmart_candidate_bio: candidate_record.votesmart_candidate_bio,
                    votesmart_candidate_ratings: candidate_record.votesmart_candidate_ratings,
                    upcoming_race_id: candidate_record.upcoming_race_id,
                    created_at: candidate_record.created_at,
                    updated_at: candidate_record.updated_at,
                };

                CandidateResult {
                    politician: politician_record.into(),
                    is_running: candidate_record.is_running,
                    date_announced: candidate_record.date_announced,
                    date_qualified: candidate_record.date_qualified,
                    date_dropped: candidate_record.date_dropped,
                    reason_dropped: candidate_record.reason_dropped,
                    qualification_method: candidate_record.qualification_method,
                    qualification_info: candidate_record.qualification_info,
                }
            })
            .collect();

        Ok(candidate_results)
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
