use crate::{context::ApiContext, types::PoliticianResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{
        enums::{FullState, PoliticalParty, PoliticalScope, State},
        office::Office,
        politician::Politician,
    },
    Chamber, District, ElectionScope,
};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OfficeResult {
    id: ID,
    slug: String,
    title: String,
    name: Option<String>,
    office_type: Option<String>,
    district: Option<String>,
    district_type: Option<District>,
    hospital_district: Option<String>,
    school_district: Option<String>,
    political_scope: PoliticalScope,
    election_scope: ElectionScope,
    chamber: Option<Chamber>,
    state: Option<State>,
    county: Option<String>,
    municipality: Option<String>,
    term_length: Option<i32>,
    seat: Option<String>,
}

#[ComplexObject]
impl OfficeResult {
    async fn subtitle(&self) -> Result<Option<String>> {
        let subtitle = match self.election_scope {
            ElectionScope::National => None,
            ElectionScope::State => self
                .state
                .as_ref()
                .map(|state| state.full_state().to_string()),
            ElectionScope::District => {
                if let Some(district_type) = self.district_type {
                    match district_type {
                        District::UsCongressional => {
                            if let (Some(state), Some(district)) = (&self.state, &self.district) {
                                Some(format!("{} District {}", state.full_state(), district))
                            } else {
                                None
                            }
                        }
                        District::StateSenate => {
                            if let (Some(state), Some(district)) = (&self.state, &self.district) {
                                Some(format!(
                                    "{} Senate District {}",
                                    state.full_state(),
                                    district
                                ))
                            } else {
                                None
                            }
                        }
                        District::StateHouse => {
                            if let (Some(state), Some(district)) = (&self.state, &self.district) {
                                Some(format!(
                                    "{} House District {}",
                                    state.full_state(),
                                    district
                                ))
                            } else {
                                None
                            }
                        }
                        District::County => {
                            if let (Some(muni), Some(district)) =
                                (&self.municipality, &self.district)
                            {
                                Some(format!("{} County District {}", muni, district))
                            } else {
                                None
                            }
                        }
                        District::City => {
                            if let (Some(muni), Some(district)) =
                                (&self.municipality, &self.district)
                            {
                                Some(format!("{} District {}", muni, district))
                            } else {
                                None
                            }
                        }
                        District::School => {
                            if let (Some(muni), Some(district)) =
                                (&self.municipality, &self.district)
                            {
                                Some(format!("{} School District {}", muni, district))
                            } else {
                                None
                            }
                        }
                        District::Hospital => {
                            if let (Some(muni), Some(district)) =
                                (&self.municipality, &self.district)
                            {
                                Some(format!("{} Hospital District {}", muni, district))
                            } else {
                                None
                            }
                        }
                        District::Judicial => {
                            if let (Some(muni), Some(district)) =
                                (&self.municipality, &self.district)
                            {
                                Some(format!("{} Judicial District {}", muni, district))
                            } else {
                                None
                            }
                        }
                        District::SoilAndWater => {
                            if let (Some(muni), Some(district)) =
                                (&self.municipality, &self.district)
                            {
                                Some(format!("{} Soil and Water District {}", muni, district))
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            }
            ElectionScope::County => self
                .municipality
                .as_ref()
                .map(|muni| format!("{} County", muni)),
            ElectionScope::City => self.municipality.as_ref().map(|muni| muni.to_string()),
        };
        Ok(subtitle)
    }

    async fn subtitle_short(&self) -> Result<Option<String>> {
        todo!(
            "Implement subtitle_short, reuse above logic but truncate state names and HD, SD, etc"
        );
    }

    async fn incumbent(&self, ctx: &Context<'_>) -> Result<Option<PoliticianResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
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
                        created_at,
                        updated_at FROM politician
                WHERE office_id = $1
            "#,
            uuid::Uuid::parse_str(&self.id.as_str()).unwrap()
        )
        .fetch_optional(&db_pool)
        .await?;

        if let Some(record) = record {
            let politician_result = PoliticianResult::from(record);
            Ok(Some(politician_result))
        } else {
            Ok(None)
        }
    }
}

impl From<Office> for OfficeResult {
    fn from(o: Office) -> Self {
        Self {
            id: ID::from(o.id),
            slug: o.slug,
            title: o.title,
            name: o.name,
            office_type: o.office_type,
            district: o.district,
            district_type: o.district_type,
            hospital_district: o.hospital_district,
            school_district: o.school_district,
            chamber: o.chamber,
            political_scope: o.political_scope,
            election_scope: o.election_scope,
            state: o.state,
            county: o.county,
            municipality: o.municipality,
            term_length: o.term_length,
            seat: o.seat,
        }
    }
}
