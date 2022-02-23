use super::{
    votesmart::VsRating, BillResult, IssueTagResult, OfficeResult, OrganizationResult, RaceResult,
};
use crate::relay;
use async_graphql::{ComplexObject, Context, Enum, FieldResult, SimpleObject, ID};
use db::{
    models::{
        bill::Bill,
        enums::{LegislationStatus, PoliticalParty, State},
        politician::Politician,
    },
    DateTime, Office, Race,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use votesmart::GetCandidateBioResponse;

use chrono::Datelike;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum OfficeType {
    House,
    Senate,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct PoliticianResult {
    id: ID,
    slug: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    nickname: Option<String>,
    preferred_name: Option<String>,
    ballot_name: Option<String>,
    description: Option<String>,
    home_state: State,
    office_id: Option<ID>,
    thumbnail_image_url: Option<String>,
    website_url: Option<String>,
    twitter_url: Option<String>,
    facebook_url: Option<String>,
    instagram_url: Option<String>,
    office_party: Option<PoliticalParty>,
    votesmart_candidate_id: i32,
    votesmart_candidate_bio: GetCandidateBioResponse,
    votesmart_candidate_ratings: Vec<VsRating>,
    upcoming_race_id: Option<ID>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct SponsoredBillResult {
    id: ID,
    title: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Endorsements {
    politicians: Vec<PoliticianResult>,
    organizations: Vec<OrganizationResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Experience {
    span: String,
    title: String,
    special: Option<String>,
    district: Option<String>,
    full_text: Option<String>,
    organization: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VotesmartExperience {
    Object(Experience),
    Array(Vec<Experience>),
    None,
}

#[ComplexObject]
impl PoliticianResult {
    async fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle_name) => format!(
                "{} {} {}",
                &self.first_name,
                middle_name.to_string(),
                &self.last_name
            ),
            None => format!("{} {}", &self.first_name, &self.last_name),
        }
    }

    /// Calculates the total years a politician has been in office using
    /// the votesmart politicial experience array.  Does not take into account
    /// objects where the politician is considered a 'candidate'
    async fn years_in_public_office(&self) -> FieldResult<i32> {
        let experience: VotesmartExperience = serde_json::from_value(
            self.votesmart_candidate_bio.candidate.political["experience"].to_owned(),
        )
        .unwrap();
        match experience {
            VotesmartExperience::Object(exp) => {
                let years = exp.span.split("-").collect::<Vec<&str>>();
                let start_year = years[0].parse::<i32>().unwrap();
                let end_year = years[1]
                    .parse::<i32>()
                    .unwrap_or(chrono::Local::now().year());
                let years_in_public_office = (end_year - start_year).abs();
                Ok(years_in_public_office)
            }
            VotesmartExperience::Array(exp_vec) => {
                let years_in_office = exp_vec.into_iter().fold(0, |acc, x| {
                    if x.title != "Candidate".to_string() {
                        let span = x
                            .span
                            .split("-")
                            // Sometimes span goes to 'present' so we need to convert that to current year
                            .map(|n| n.parse::<i32>().unwrap_or(chrono::Utc::now().year()))
                            .collect::<Vec<i32>>();
                        if span.len() == 1 {
                            acc + (chrono::Utc::now().year() - span[0]).abs()
                        } else {
                            acc + (span[1] - span[0]).abs()
                        }
                    } else {
                        acc
                    }
                });

                Ok(years_in_office)
            }
            VotesmartExperience::None => Ok(0),
        }
    }

    async fn endorsements(&self, ctx: &Context<'_>) -> FieldResult<Endorsements> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();

        let mut politician_results: Vec<PoliticianResult> = vec![];
        let mut organization_results: Vec<OrganizationResult> = vec![];

        if ctx.look_ahead().field("organizations").exists() {
            let organization_records = Politician::organization_endorsements(
                db_pool,
                uuid::Uuid::parse_str(&self.id).unwrap(),
            )
            .await?;
            organization_results = organization_records
                .into_iter()
                .map(OrganizationResult::from)
                .collect();
        }

        if ctx.look_ahead().field("politicians").exists() {
            let politician_records = Politician::politician_endorsements(
                db_pool,
                uuid::Uuid::parse_str(&self.id).unwrap(),
            )
            .await?;
            politician_results = politician_records
                .into_iter()
                .map(PoliticianResult::from)
                .collect();
        }

        Ok(Endorsements {
            politicians: politician_results,
            organizations: organization_results,
        })
    }

    async fn issue_tags(&self, ctx: &Context<'_>) -> FieldResult<Vec<IssueTagResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records =
            Politician::issue_tags(pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(IssueTagResult::from).collect();
        Ok(results)
    }

    async fn sponsored_bills(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<BillResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = sqlx::query_as!(
            Bill,
            r#"
                SELECT id, slug, title, bill_number, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, history, votesmart_bill_id, created_at, updated_at FROM bill, jsonb_array_elements(legiscan_data->'sponsors') sponsors 
                WHERE sponsors->>'votesmart_id' = $1
                LIMIT 25
            "#,
            &self.votesmart_candidate_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let results = records.into_iter().map(BillResult::from);
        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    pub async fn current_office(&self, ctx: &Context<'_>) -> FieldResult<Option<OfficeResult>> {
        let office_result = match &self.office_id {
            Some(id) => {
                let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
                let office =
                    Office::find_by_id(db_pool, uuid::Uuid::parse_str(id).unwrap()).await?;
                Some(OfficeResult::from(office))
            }
            None => None,
        };

        Ok(office_result)
    }

    async fn upcoming_race(&self, ctx: &Context<'_>) -> FieldResult<Option<RaceResult>> {
        let race_result = match &self.upcoming_race_id {
            Some(id) => {
                let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
                let race = Race::find_by_id(db_pool, uuid::Uuid::parse_str(id).unwrap()).await?;
                Some(RaceResult::from(race))
            }
            None => None,
        };

        Ok(race_result)
    }
}

impl From<Politician> for PoliticianResult {
    fn from(p: Politician) -> Self {
        Self {
            id: ID::from(p.id),
            slug: p.slug,
            first_name: p.first_name,
            middle_name: p.middle_name,
            last_name: p.last_name,
            nickname: p.nickname,
            preferred_name: p.preferred_name,
            ballot_name: p.ballot_name,
            description: p.description,
            home_state: p.home_state,
            office_id: match p.office_id {
                Some(id) => Some(ID::from(id)),
                None => None,
            },
            thumbnail_image_url: p.thumbnail_image_url,
            website_url: p.website_url,
            twitter_url: p.twitter_url,
            facebook_url: p.facebook_url,
            instagram_url: p.instagram_url,
            office_party: p.office_party,
            votesmart_candidate_id: p.votesmart_candidate_id.unwrap(),
            votesmart_candidate_bio: serde_json::from_value(p.votesmart_candidate_bio.to_owned())
                .unwrap_or_default(),
            votesmart_candidate_ratings: serde_json::from_value(
                p.votesmart_candidate_ratings.to_owned(),
            )
            .unwrap_or_default(),
            upcoming_race_id: match p.upcoming_race_id {
                Some(id) => Some(ID::from(id)),
                None => None,
            },
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
