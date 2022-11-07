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
    subtitle: Option<String>,
    subtitle_short: Option<String>,
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
    priority: Option<i32>,
}

fn compute_office_subtitle(office: &Office, use_short: bool) -> Option<String> {
    match office.election_scope {
        ElectionScope::National => None,
        ElectionScope::State => office
            .state
            .as_ref()
            .map(|state| state.full_state().to_string()),
        ElectionScope::District => {
            if let Some(district_type) = office.district_type {
                match district_type {
                    District::UsCongressional => {
                        if let (Some(state), Some(district)) = (&office.state, &office.district) {
                            Some(format!("{} District {}", state.full_state(), district))
                        } else {
                            None
                        }
                    }
                    District::StateSenate => {
                        if let (Some(state), Some(district)) = (&office.state, &office.district) {
                            Some(format!(
                                "{} {} {}",
                                state.full_state(),
                                if use_short { "SD" } else { "Senate District" },
                                district
                            ))
                        } else {
                            None
                        }
                    }
                    District::StateHouse => {
                        if let (Some(state), Some(district)) = (&office.state, &office.district) {
                            Some(format!(
                                "{} {} {}",
                                state.full_state(),
                                if use_short { "HD" } else { "House District" },
                                district
                            ))
                        } else {
                            None
                        }
                    }
                    District::County => {
                        if let (Some(county), Some(district)) = (&office.county, &office.district) {
                            Some(format!("{} County District {}", county, district))
                        } else {
                            None
                        }
                    }
                    District::City => {
                        if let (Some(muni), Some(district)) =
                            (&office.municipality, &office.district)
                        {
                            Some(format!("{} District {}", muni, district))
                        } else {
                            None
                        }
                    }
                    District::School => {
                        if let (Some(muni), Some(district)) =
                            (&office.municipality, &office.district)
                        {
                            Some(format!("{} School District {}", muni, district))
                        } else {
                            None
                        }
                    }
                    District::Hospital => {
                        if let (Some(muni), Some(district)) =
                            (&office.municipality, &office.district)
                        {
                            Some(format!("{} Hospital District {}", muni, district))
                        } else {
                            None
                        }
                    }
                    District::Judicial => {
                        if let (Some(muni), Some(district)) =
                            (&office.municipality, &office.district)
                        {
                            Some(format!("{} Judicial District {}", muni, district))
                        } else {
                            None
                        }
                    }
                    District::SoilAndWater => {
                        if let (Some(muni), Some(district)) =
                            (&office.municipality, &office.district)
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
        ElectionScope::County => office
            .county
            .as_ref()
            .map(|county| format!("{} County", county)),
        ElectionScope::City => office.municipality.as_ref().map(|muni| muni.to_string()),
    }
}

#[ComplexObject]
impl OfficeResult {
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
        let subtitle = if o.subtitle.is_none() || o.subtitle.clone().unwrap().is_empty() {
            compute_office_subtitle(&o, false)
        } else {
            o.clone().subtitle
        };

        let subtitle_short = if o.subtitle_short.is_none() {
            compute_office_subtitle(&o, true)
        } else {
            o.subtitle_short
        };

        Self {
            id: ID::from(o.id),
            slug: o.slug,
            title: o.title,
            subtitle,
            subtitle_short,
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
            priority: o.priority,
        }
    }
}

#[tokio::test]
async fn test_compute_office_title() {
    let office = Office {
        id: uuid::Uuid::new_v4(),
        slug: "test-state-senator".to_string(),
        title: "State Senator".to_string(),
        subtitle: None,
        subtitle_short: None,
        name: None,
        office_type: None,
        district: Some("1".to_string()),
        district_type: Some(District::StateSenate),
        hospital_district: None,
        school_district: None,
        chamber: Some(Chamber::Senate),
        political_scope: PoliticalScope::State,
        election_scope: ElectionScope::District,
        state: Some(State::AL),
        county: Some("Buckwild County".to_string()),
        municipality: None,
        term_length: None,
        seat: None,
        ..Default::default()
    };

    assert_eq!(
        compute_office_subtitle(&office, false),
        Some("Alabama Senate District 1".to_string())
    );
    assert_eq!(
        compute_office_subtitle(&office, true),
        Some("Alabama SD 1".to_string())
    );
}
