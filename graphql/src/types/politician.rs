use super::{
    votesmart::VsRating, BillResult, IssueTagResult, OfficeResult, OrganizationResult, RaceResult,
};
use crate::{context::ApiContext, relay};
use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject, ID};
use db::{
    models::{
        bill::Bill,
        enums::{LegislationStatus, PoliticalParty, State},
        politician::Politician,
    },
    DateTime, Office, Race,
};
use serde::{Deserialize, Serialize};

use votesmart::GetCandidateBioResponse;

use chrono::{Datelike, Local, NaiveDate};

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
    home_state: Option<State>,
    office_id: Option<ID>,
    thumbnail_image_url: Option<String>,
    website_url: Option<String>,
    twitter_url: Option<String>,
    facebook_url: Option<String>,
    instagram_url: Option<String>,
    party: Option<PoliticalParty>,
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

#[derive(SimpleObject, Debug, Clone)]
pub struct RatingResult {
    vs_rating: VsRating,
    organization: Option<OrganizationResult>,
}

#[ComplexObject]
impl PoliticianResult {
    async fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle_name) => {
                format!("{} {} {}", &self.first_name, middle_name, &self.last_name)
            }
            None => format!("{} {}", &self.first_name, &self.last_name),
        }
    }

    async fn age(&self) -> Option<i64> {
        let dob = NaiveDate::parse_from_str(
            &self.votesmart_candidate_bio.candidate.birth_date,
            "%m/%d/%Y",
        );
        // Votesmart dob may be in a whack format
        if let Ok(dob) = dob {
            // There must be a better way to get NaiveDate.today but 🤷
            let now =
                NaiveDate::parse_from_str(&Local::now().format("%m/%d/%Y").to_string(), "%m/%d/%Y")
                    .unwrap();
            let age = (now - dob).num_weeks() / 52;
            Some(age)
        } else {
            None
        }
    }

    /// Leverages Votesmart ratings data for the time being
    async fn ratings(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<RatingResult> {
        let mut ratings = vec![];
        // let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let unique_sig_ids = self
            .votesmart_candidate_ratings
            .iter()
            .map(|rating| rating.sig_id.as_str().unwrap().parse::<i32>().unwrap())
            .collect::<Vec<i32>>()
            .into_iter()
            .collect::<std::collections::HashSet<i32>>();

        // Preload all organizations to avoid expensive n + 1
        let organizations = ctx
            .data::<ApiContext>()?
            .loaders
            .organization_loader
            .load_many(unique_sig_ids)
            .await?;

        self.votesmart_candidate_ratings
            .iter()
            .for_each(|vs_rating| {
                let sig_id = vs_rating.sig_id.as_str().unwrap().parse::<i32>().unwrap();
                let organization = organizations.get(&sig_id).to_owned();

                let rating = RatingResult {
                    vs_rating: vs_rating.to_owned(),
                    organization: organization.map(|org| OrganizationResult::from(org.to_owned())),
                };
                ratings.push(rating);
            });

        relay::query(
            ratings.into_iter(),
            relay::Params::new(after, before, first, last),
            25,
        )
        .await
    }

    /// Calculates the total years a politician has been in office using
    /// the votesmart politicial experience array.  Does not take into account
    /// objects where the politician is considered a 'candidate'
    async fn years_in_public_office(&self) -> Result<i32> {
        let experience: VotesmartExperience = serde_json::from_value(
            self.votesmart_candidate_bio.candidate.political["experience"].to_owned(),
        )
        .unwrap();
        match experience {
            VotesmartExperience::Object(exp) => {
                let years = exp.span.split('-').collect::<Vec<&str>>();
                let start_year = years[0].parse::<i32>().unwrap();
                let end_year = years[1]
                    .parse::<i32>()
                    .unwrap_or_else(|_| chrono::Local::now().year());
                let years_in_public_office = (end_year - start_year).abs();
                Ok(years_in_public_office)
            }
            VotesmartExperience::Array(exp_vec) => {
                let years_in_office = exp_vec.into_iter().fold(0, |acc, x| {
                    if x.title != "Candidate" {
                        let span = x
                            .span
                            .split('-')
                            // Sometimes span goes to 'present' so we need to convert that to current year
                            .map(|n| {
                                n.parse::<i32>()
                                    .unwrap_or_else(|_| chrono::Utc::now().year())
                            })
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

    async fn endorsements(&self, ctx: &Context<'_>) -> Result<Endorsements> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let mut politician_results: Vec<PoliticianResult> = vec![];
        let mut organization_results: Vec<OrganizationResult> = vec![];

        if ctx.look_ahead().field("organizations").exists() {
            let organization_records = Politician::organization_endorsements(
                &db_pool,
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
                &db_pool,
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

    async fn issue_tags(&self, ctx: &Context<'_>) -> Result<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records =
            Politician::issue_tags(&db_pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
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
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = sqlx::query_as!(
            Bill,
            r#"
                SELECT id, slug, title, bill_number, legislation_status AS "legislation_status:LegislationStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, history, votesmart_bill_id, b.created_at, b.updated_at FROM bill b
                JOIN bill_sponsors 
                ON bill_sponsors.bill_id = id
                WHERE bill_sponsors.politician_id = $1
            "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;

        let results = records.into_iter().map(BillResult::from);
        relay::query(results, relay::Params::new(after, before, first, last), 10).await
    }

    pub async fn current_office(&self, ctx: &Context<'_>) -> Result<Option<OfficeResult>> {
        let office_result = match &self.office_id {
            Some(id) => {
                let db_pool = ctx.data::<ApiContext>()?.pool.clone();
                let office =
                    Office::find_by_id(&db_pool, uuid::Uuid::parse_str(id).unwrap()).await?;
                Some(OfficeResult::from(office))
            }
            None => None,
        };

        Ok(office_result)
    }

    async fn upcoming_race(&self, ctx: &Context<'_>) -> Result<Option<RaceResult>> {
        let race_result = match &self.upcoming_race_id {
            Some(id) => {
                let db_pool = ctx.data::<ApiContext>()?.pool.clone();
                let race = Race::find_by_id(&db_pool, uuid::Uuid::parse_str(id).unwrap()).await?;
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
            office_id: p.office_id.map(ID::from),
            thumbnail_image_url: p.thumbnail_image_url,
            website_url: p.website_url,
            twitter_url: p.twitter_url,
            facebook_url: p.facebook_url,
            instagram_url: p.instagram_url,
            party: p.party,
            votesmart_candidate_id: p.votesmart_candidate_id.unwrap(),
            votesmart_candidate_bio: serde_json::from_value(p.votesmart_candidate_bio.to_owned())
                .unwrap_or_default(),
            votesmart_candidate_ratings: serde_json::from_value(
                p.votesmart_candidate_ratings.to_owned(),
            )
            .unwrap_or_default(),
            upcoming_race_id: p.upcoming_race_id.map(ID::from),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
