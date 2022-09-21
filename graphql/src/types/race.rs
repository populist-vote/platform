use crate::{context::ApiContext, types::OfficeResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{
        enums::{PoliticalParty, RaceType, State},
        politician::Politician,
        race::Race,
    },
    Office,
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
    official_website: Option<String>,
    election_id: Option<ID>,
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
    winner: Option<PoliticianResult>,
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
                    suffix,
                    preferred_name,
                    biography,
                    biography_source,
                    home_state AS "home_state:State",
                    date_of_birth,
                    office_id,
                    thumbnail_image_url,
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
                    party AS "party:PoliticalParty",
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
              winner_id
            FROM
              race
            WHERE
              id = $1
        "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_one(&db_pool)
        .await?;

        let winner = match ctx.look_ahead().field("winner").exists() {
            true => match race_results.winner_id {
                Some(winner_id) => Some(PoliticianResult::from(
                    Politician::find_by_id(&db_pool, winner_id).await?,
                )),
                _ => None,
            },
            false => None,
        };

        Ok(RaceResultsResult {
            votes_by_candidate: race_candidate_results,
            total_votes: race_results.total_votes,
            winner,
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
}

#[ComplexObject]
impl RaceCandidateResult {
    async fn vote_percentage(&self, ctx: &Context<'_>) -> Result<Option<f64>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query!(
            r#"
            SELECT
                ROUND(CAST(CAST(rc.votes AS FLOAT) / CAST(r.total_votes AS FLOAT) * 100 AS NUMERIC), 1)::FLOAT AS "percentage"
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
            party: r.party,
            state: r.state,
            description: r.description,
            ballotpedia_link: r.ballotpedia_link,
            early_voting_begins_date: r.early_voting_begins_date,
            official_website: r.official_website,
            election_id: r.election_id.map(ID::from),
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
