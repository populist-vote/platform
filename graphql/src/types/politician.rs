use async_graphql::{ComplexObject, Context, Enum, FieldResult, SimpleObject, ID};
use db::{
    models::{
        bill::Bill,
        enums::{LegislationStatus, PoliticalParty, State},
        politician::Politician,
    },
    DateTime,
};

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use votesmart::GetCandidateBioResponse;

use super::{BillResult, IssueTagResult, OrganizationResult};

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
    thumbnail_image_url: Option<String>,
    website_url: Option<String>,
    twitter_url: Option<String>,
    facebook_url: Option<String>,
    instagram_url: Option<String>,
    office_party: Option<PoliticalParty>,
    votesmart_candidate_id: i32,
    votesmart_candidate_bio: GetCandidateBioResponse,
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
enum ExperienceResponse {
    Object(Experience),
    Array(Vec<Experience>),
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
        let experience: ExperienceResponse = serde_json::from_value(
            self.votesmart_candidate_bio.candidate.political["experience"].to_owned(),
        )
        .unwrap();
        match experience {
            ExperienceResponse::Object(exp) => {
                let years = exp.span.split("-").collect::<Vec<&str>>();
                let start_year = years[0].parse::<i32>().unwrap();
                let end_year = years[1]
                    .parse::<i32>()
                    .unwrap_or(chrono::Local::now().year());
                let years_in_public_office = (end_year - start_year).abs();
                Ok(years_in_public_office)
            }
            ExperienceResponse::Array(exp_vec) => {
                let years_in_office = exp_vec.into_iter().fold(0, |acc, x| {
                    if x.title != "Candidate".to_string() {
                        let span = x
                            .span
                            .split("-")
                            // Sometimes span goes to 'present' so we need to convert that to current year
                            .map(|n| n.parse::<i32>().unwrap_or(chrono::Utc::now().year()))
                            .collect::<Vec<i32>>();
                        acc + (span[0] - span[1]).abs()
                    } else {
                        acc
                    }
                });

                Ok(years_in_office)
            }
        }
    }

    async fn endorsements(&self, ctx: &Context<'_>) -> FieldResult<Endorsements> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();

        let mut politician_results: Vec<PoliticianResult> = vec![];
        let mut organization_results: Vec<OrganizationResult> = vec![];

        if ctx.look_ahead().field("organizations").exists() {
            let organization_records = Politician::organization_endorsements(
                pool,
                uuid::Uuid::parse_str(&self.id).unwrap(),
            )
            .await?;
            organization_results = organization_records
                .into_iter()
                .map(OrganizationResult::from)
                .collect();
        }

        if ctx.look_ahead().field("politicians").exists() {
            println!("TRUE ITS HERE");
            let politician_records =
                Politician::politician_endorsements(pool, uuid::Uuid::parse_str(&self.id).unwrap())
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

    async fn sponsored_bills(&self, ctx: &Context<'_>) -> FieldResult<Vec<BillResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = sqlx::query_as!(
            Bill,
            r#"
                SELECT id, slug, title, bill_number, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, history, votesmart_bill_id, created_at, updated_at FROM bill, jsonb_array_elements(legiscan_data->'sponsors') sponsors 
                WHERE sponsors->>'votesmart_id' = $1
            "#,
            &self.votesmart_candidate_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let results = records.into_iter().map(BillResult::from).collect();
        Ok(results)
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
            thumbnail_image_url: p.thumbnail_image_url,
            website_url: p.website_url,
            twitter_url: p.twitter_url,
            facebook_url: p.facebook_url,
            instagram_url: p.instagram_url,
            office_party: p.office_party,
            votesmart_candidate_id: p.votesmart_candidate_id.unwrap(),
            votesmart_candidate_bio: serde_json::from_value(p.votesmart_candidate_bio.to_owned())
                .unwrap(),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
