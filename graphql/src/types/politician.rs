use super::{
    votesmart::VsRating, BillResult, IssueTagResult, OfficeResult, OrganizationResult, RaceResult,
};
use crate::{context::ApiContext, relay};
use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject, ID};
use db::{
    models::{
        bill::Bill,
        enums::{LegislationStatus, PoliticalParty, RaceType, State},
        politician::Politician,
    },
    Office, Race,
};
use open_secrets::OpenSecretsProxy;
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
    suffix: Option<String>,
    preferred_name: Option<String>,
    biography: Option<String>,
    biography_source: Option<String>,
    home_state: Option<State>,
    date_of_birth: Option<NaiveDate>,
    office_id: Option<ID>,
    thumbnail_image_url: Option<String>,
    assets: Assets,
    official_website_url: Option<String>,
    campaign_website_url: Option<String>,
    facebook_url: Option<String>,
    twitter_url: Option<String>,
    instagram_url: Option<String>,
    youtube_url: Option<String>,
    linkedin_url: Option<String>,
    tiktok_url: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    party: Option<PoliticalParty>,
    crp_candidate_id: Option<String>,
    votesmart_candidate_id: Option<i32>,
    votesmart_candidate_bio: Option<GetCandidateBioResponse>,
    votesmart_candidate_ratings: Vec<VsRating>,
    race_wins: Option<i32>,
    race_losses: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Assets {
    thumbnail_image_160: Option<String>,
    thumbnail_image_400: Option<String>,
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
pub struct DonationsSummary {
    total_raised: f64,
    spent: f64,
    cash_on_hand: f64,
    debt: f64,
    source: String,
    last_updated: NaiveDate,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct DonationsByIndustry {
    cycle: i32,
    last_updated: chrono::NaiveDate,
    sectors: Vec<Sector>,
    source: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Sector {
    name: String,
    id: String,
    individuals: i32,
    pacs: i32,
    total: i32,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct RatingResult {
    vs_rating: VsRating,
    organization: Option<OrganizationResult>,
}

fn calculate_age(dob: NaiveDate) -> Result<i64> {
    let now = NaiveDate::parse_from_str(&Local::now().format("%m/%d/%Y").to_string(), "%m/%d/%Y")
        .unwrap();
    let age = (now - dob).num_days() / 365;
    Ok(age)
}

#[test]
fn test_calculate_age() {
    let dob = NaiveDate::parse_from_str("05/13/1984", "%m/%d/%Y").unwrap();
    assert_eq!(calculate_age(dob), Ok(38));

    let dob = NaiveDate::parse_from_str("02/09/1992", "%m/%d/%Y").unwrap();
    assert_eq!(calculate_age(dob), Ok(30));
}

async fn fetch_donations_summary(crp_id: String) -> Result<DonationsSummary> {
    let proxy = OpenSecretsProxy::new()?;
    let response = proxy.cand_summary(&crp_id, None).await?;
    let json: serde_json::Value = response.json().await?;
    let donations = DonationsSummary {
        total_raised: json["response"]["summary"]["@attributes"]["total"]
            .as_str()
            .unwrap_or_default()
            .parse::<f64>()
            .unwrap_or_default(),
        spent: json["response"]["summary"]["@attributes"]["spent"]
            .as_str()
            .unwrap_or_default()
            .parse::<f64>()
            .unwrap_or_default(),
        cash_on_hand: json["response"]["summary"]["@attributes"]["cash_on_hand"]
            .as_str()
            .unwrap_or_default()
            .parse::<f64>()
            .unwrap_or_default(),
        debt: json["response"]["summary"]["@attributes"]["debt"]
            .as_str()
            .unwrap_or_default()
            .parse::<f64>()
            .unwrap_or_default(),
        source: json["response"]["summary"]["@attributes"]["source"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        last_updated: chrono::NaiveDate::parse_from_str(
            json["response"]["summary"]["@attributes"]["last_updated"]
                .as_str()
                .unwrap_or_default(),
            "%m/%d/%Y",
        )
        .unwrap_or_default(),
    };
    Ok(donations)
}

async fn fetch_donations_by_industry(crp_id: String) -> Result<DonationsByIndustry> {
    let proxy = OpenSecretsProxy::new()?;
    let response = proxy.cand_sector(&crp_id, None).await?;
    let json: serde_json::Value = response.json().await?;
    let donations_by_industry = DonationsByIndustry {
        cycle: json["response"]["sectors"]["@attributes"]["cycle"]
            .as_str()
            .unwrap()
            .parse::<i32>()
            .unwrap(),
        last_updated: chrono::NaiveDate::parse_from_str(
            json["response"]["sectors"]["@attributes"]["last_updated"]
                .as_str()
                .unwrap(),
            "%m/%d/%Y",
        )
        .unwrap(),
        sectors: json["response"]["sectors"]["sector"]
            .as_array()
            .unwrap()
            .iter()
            .map(|sector| {
                let name = sector["@attributes"]["sector_name"]
                    .as_str()
                    .unwrap()
                    .to_string();
                let id = sector["@attributes"]["sectorid"]
                    .as_str()
                    .unwrap()
                    .to_string();
                let individuals = sector["@attributes"]["indivs"]
                    .as_str()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();
                let pacs = sector["@attributes"]["pacs"]
                    .as_str()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();
                let total = sector["@attributes"]["total"]
                    .as_str()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();
                Sector {
                    name,
                    id,
                    individuals,
                    pacs,
                    total,
                }
            })
            .collect(),
        source: json["response"]["sectors"]["@attributes"]["source"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
    };
    Ok(donations_by_industry)
}

#[ComplexObject]
impl PoliticianResult {
    async fn full_name(&self) -> String {
        format!(
            "{first_name} {last_name} {suffix}",
            first_name = &self.preferred_name.as_ref().unwrap_or(&self.first_name),
            last_name = &self.last_name,
            suffix = &self.suffix.as_ref().unwrap_or(&"".to_string())
        )
        .trim_end()
        .to_string()
    }

    async fn age(&self) -> Option<i64> {
        match self.date_of_birth {
            Some(dob) => calculate_age(dob).ok(),
            None => None,
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
    async fn years_in_public_office(&self) -> Result<Option<i32>> {
        if let Some(vs_bio) = &self.votesmart_candidate_bio {
            let experience: VotesmartExperience =
                serde_json::from_value(vs_bio.candidate.political["experience"].to_owned())
                    .unwrap();
            match experience {
                VotesmartExperience::Object(exp) => {
                    let years = exp.span.split('-').collect::<Vec<&str>>();
                    let start_year = years[0].parse::<i32>().unwrap();
                    let end_year = years[1]
                        .parse::<i32>()
                        .unwrap_or_else(|_| chrono::Local::now().year());
                    let years_in_public_office = (end_year - start_year).abs();
                    Ok(Some(years_in_public_office))
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

                    Ok(Some(years_in_office))
                }
                VotesmartExperience::None => Ok(Some(0)),
            }
        } else {
            Ok(None)
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
                let cached_office = ctx
                    .data::<ApiContext>()?
                    .loaders
                    .office_loader
                    .load_one(uuid::Uuid::parse_str(id).unwrap())
                    .await?;

                if let Some(office) = cached_office {
                    Some(OfficeResult::from(office))
                } else {
                    let db_pool = ctx.data::<ApiContext>()?.pool.clone();
                    let record =
                        Office::find_by_id(&db_pool, uuid::Uuid::parse_str(id).unwrap()).await?;
                    Some(record.into())
                }
            }
            None => None,
        };

        Ok(office_result)
    }

    pub async fn donations_summary(&self) -> Result<Option<DonationsSummary>> {
        if let Some(crp_id) = &self.crp_candidate_id {
            let donations_summary = match fetch_donations_summary(crp_id.into()).await {
                Ok(summary) => Some(summary),
                Err(_) => None,
            };
            Ok(donations_summary)
        } else {
            Ok(None)
        }
    }

    pub async fn donations_by_industry(&self) -> Result<Option<DonationsByIndustry>> {
        if let Some(crp_id) = &self.crp_candidate_id {
            let donations_by_industry = match fetch_donations_by_industry(crp_id.into()).await {
                Ok(donations) => Some(donations),
                Err(_) => None,
            };
            Ok(donations_by_industry)
        } else {
            Ok(None)
        }
    }

    async fn upcoming_race(&self, ctx: &Context<'_>) -> Result<Option<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let race_result = sqlx::query_as!(
            Race,
            r#"
            SELECT
                id,
                slug,
                title,
                office_id,
                race_type AS "race_type:RaceType",
                party AS "party:PoliticalParty",
                state AS "state:State",
                description,
                ballotpedia_link,
                early_voting_begins_date,
                winner_id,
                total_votes,
                official_website,
                election_id,
                is_special_election,
                num_elect,
                created_at,
                updated_at
            FROM (
                SELECT
                    race.*,
                    election_date
                FROM
                    race
                    JOIN election ON race.election_id = election.id
                    JOIN race_candidates ON race.id = race_candidates.race_id
                WHERE
                    race_candidates.candidate_id = $1
                    AND election_date > NOW()) AS r
            ORDER BY
                election_date ASC
            LIMIT 1
        "#,
            uuid::Uuid::parse_str(&self.id).unwrap(),
        )
        .fetch_optional(&db_pool)
        .await?;

        Ok(race_result.map(RaceResult::from))
    }

    async fn votes(&self, ctx: &Context<'_>, race_id: uuid::Uuid) -> Result<Option<i32>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query!(
            r#"
            SELECT
                votes
            FROM
                race_candidates
            WHERE
                race_candidates.candidate_id = $1 AND
                race_candidates.race_id = $2
        "#,
            uuid::Uuid::parse_str(&self.id).unwrap(),
            race_id
        )
        .fetch_optional(&db_pool)
        .await?;

        if let Some(record) = record {
            match record.votes {
                Some(votes) => Ok(Some(votes)),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
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
            suffix: p.suffix,
            preferred_name: p.preferred_name,
            biography: p.biography,
            biography_source: p.biography_source,
            home_state: p.home_state,
            date_of_birth: p.date_of_birth,
            office_id: p.office_id.map(ID::from),
            thumbnail_image_url: p.thumbnail_image_url,
            assets: serde_json::from_value(p.assets.to_owned()).unwrap_or_default(),
            official_website_url: p.official_website_url,
            campaign_website_url: p.campaign_website_url,
            facebook_url: p.facebook_url,
            twitter_url: p.twitter_url,
            instagram_url: p.instagram_url,
            youtube_url: p.youtube_url,
            linkedin_url: p.linkedin_url,
            tiktok_url: p.tiktok_url,
            email: p.email,
            phone: p.phone,
            party: p.party,
            crp_candidate_id: p.crp_candidate_id,
            votesmart_candidate_id: p.votesmart_candidate_id,
            votesmart_candidate_bio: serde_json::from_value(p.votesmart_candidate_bio.to_owned())
                .unwrap_or_default(),
            votesmart_candidate_ratings: serde_json::from_value(
                p.votesmart_candidate_ratings.to_owned(),
            )
            .unwrap_or_default(),
            race_wins: p.race_wins,
            race_losses: p.race_losses,
        }
    }
}
