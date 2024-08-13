use crate::{context::ApiContext, types::OfficeResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    loaders::politician::PoliticianId,
    models::{
        enums::{RaceType, State, VoteType},
        politician::Politician,
        race::Race,
    },
    Election,
};

use super::{ElectionResult, PoliticalParty, PoliticianResult};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct RaceResult {
    id: ID,
    slug: String,
    title: String,
    office_id: ID,
    party_id: Option<ID>,
    race_type: RaceType,
    vote_type: VoteType,
    state: Option<State>,
    description: Option<String>,
    ballotpedia_link: Option<String>,
    early_voting_begins_date: Option<chrono::NaiveDate>,
    official_website: Option<String>,
    election_id: Option<ID>,
    is_special_election: bool,
    num_elect: Option<i32>,
}

pub struct RaceCandidate {
    race_id: uuid::Uuid,
    candidate_id: uuid::Uuid,
    votes: Option<i32>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct RaceCandidateResult {
    #[graphql(visible = false)]
    race_id: ID,
    candidate_id: ID,
    votes: Option<i32>,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct RaceResultsResult {
    votes_by_candidate: Vec<RaceCandidateResult>,
    total_votes: Option<i32>,
    num_precincts_reporting: Option<i32>,
    total_precincts: Option<i32>,
    precinct_reporting_percentage: Option<f64>,
    winners: Option<Vec<PoliticianResult>>,
}

#[ComplexObject]
impl RaceResult {
    async fn office(&self, ctx: &Context<'_>) -> Result<OfficeResult> {
        let office = ctx
            .data::<ApiContext>()?
            .loaders
            .office_loader
            .load_one(uuid::Uuid::parse_str(&self.office_id.clone()).unwrap())
            .await?;

        Ok(OfficeResult::from(office.unwrap()))
    }

    async fn party(&self, ctx: &Context<'_>) -> Result<Option<PoliticalParty>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let party = match &self.party_id {
            Some(party_id) => {
                let record = sqlx::query!(
                    r#"
                    SELECT id, fec_code, name, description, notes
                    FROM party
                    WHERE id = $1
                "#,
                    uuid::Uuid::parse_str(&party_id.to_string()).unwrap()
                )
                .fetch_optional(&db_pool)
                .await?;

                match record {
                    Some(record) => Some(PoliticalParty {
                        id: ID::from(record.id),
                        fec_code: record.fec_code,
                        name: record.name,
                        description: record.description,
                        notes: record.notes,
                    }),
                    None => None,
                }
            }
            None => None,
        };

        Ok(party)
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
                    suffix,
                    preferred_name,
                    biography,
                    biography_source,
                    home_state AS "home_state:State",
                    date_of_birth,
                    office_id,
                    party_id,
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

    async fn results(&self, ctx: &Context<'_>) -> Result<RaceResultsResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let race_candidate_records = sqlx::query_as!(
            RaceCandidate,
            r#"
                SELECT
                    id AS candidate_id,
                    rc.votes,
                    rc.race_id
                FROM
                    politician p
                    JOIN race_candidates rc ON race_id = $1
                WHERE
                    p.id = rc.candidate_id AND 
                    rc.race_id = $1

            "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        let race_candidate_results = race_candidate_records
            .into_iter()
            .map(RaceCandidateResult::from)
            .collect();

        let race_results = sqlx::query!(
            r#"
            SELECT
              total_votes,
              winner_ids,
              num_precincts_reporting,
              total_precincts,
              ROUND(CAST(CAST(num_precincts_reporting AS FLOAT) / CAST(NULLIF(total_precincts, 0) AS FLOAT) * 100 AS NUMERIC), 1)::FLOAT AS "precinct_reporting_percentage"
            FROM
              race
            WHERE
              id = $1
        "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_one(&db_pool)
        .await?;

        let winners = match ctx.look_ahead().field("winners").exists() {
            true => match race_results.winner_ids {
                Some(winner_ids) => {
                    let politicians = ctx
                        .data::<ApiContext>()?
                        .loaders
                        .politician_loader
                        .load_many(winner_ids.into_iter().map(PoliticianId))
                        .await?;
                    let politician_results = politicians
                        .values()
                        .cloned()
                        .collect::<Vec<Politician>>()
                        .into_iter()
                        .map(PoliticianResult::from)
                        .collect();
                    Some(politician_results)
                }
                None => None,
            },
            false => None,
        };

        Ok(RaceResultsResult {
            votes_by_candidate: race_candidate_results,
            total_votes: race_results.total_votes,
            num_precincts_reporting: race_results.num_precincts_reporting,
            total_precincts: race_results.total_precincts,
            precinct_reporting_percentage: race_results.precinct_reporting_percentage,
            winners,
        })
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

    async fn election(&self, ctx: &Context<'_>) -> Result<ElectionResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            Election,
            r#"
            SELECT id, slug, title, description, state AS "state:State", municipality, election_date 
            FROM election WHERE id = $1"#,
            uuid::Uuid::parse_str(self.election_id.clone().unwrap_or_default().as_str()).unwrap()
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(record.into())
    }
}

#[ComplexObject]
impl RaceCandidateResult {
    async fn vote_percentage(&self, ctx: &Context<'_>) -> Result<Option<f64>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query!(
            r#"
            SELECT
                ROUND(CAST(CAST(rc.votes AS FLOAT) / CAST(NULLIF(r.total_votes, 0) AS FLOAT) * 100 AS NUMERIC), 1)::FLOAT AS "percentage"
            FROM
                race_candidates rc
                JOIN race r ON rc.race_id = $2
            WHERE
                rc.candidate_id = $1
                AND rc.race_id = $2
                AND rc.votes IS NOT NULL
                AND r.id = $2
        "#,
            uuid::Uuid::parse_str(&self.candidate_id).unwrap(),
            uuid::Uuid::parse_str(&self.race_id).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        if let Some(record) = record {
            match record.percentage {
                Some(percentage) => Ok(Some(percentage)),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
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
            vote_type: r.vote_type,
            party_id: r.party_id.map(ID::from),
            state: r.state,
            description: r.description,
            ballotpedia_link: r.ballotpedia_link,
            early_voting_begins_date: r.early_voting_begins_date,
            official_website: r.official_website,
            election_id: r.election_id.map(ID::from),
            is_special_election: r.is_special_election,
            num_elect: r.num_elect,
        }
    }
}

impl From<RaceCandidate> for RaceCandidateResult {
    fn from(r: RaceCandidate) -> Self {
        Self {
            race_id: ID::from(r.race_id),
            candidate_id: ID::from(r.candidate_id),
            votes: r.votes,
        }
    }
}
