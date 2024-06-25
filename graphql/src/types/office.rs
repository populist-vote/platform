use crate::{context::ApiContext, types::PoliticianResult};
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    loaders::politician::OfficeId,
    models::{
        enums::{FullState, PoliticalScope, State},
        office::Office,
    },
    Chamber, District, ElectionScope, Politician,
};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct OfficeResult {
    id: ID,
    slug: String,
    /// What the person in office would be called, e.g. "Senator", "Governor"
    title: String,
    /// Name of the office, e.g. "U.S. Senate"
    name: Option<String>,
    subtitle: Option<String>,
    subtitle_short: Option<String>,
    office_type: Option<String>,
    /// The district name, e.g. "2, 3B, Ward 5"
    district: Option<String>,
    /// The type of district, used to determine which field is referenced for the district
    district_type: Option<District>,
    hospital_district: Option<String>,
    school_district: Option<String>,
    /// Local, State, or Federal
    political_scope: PoliticalScope,
    /// National, State, County, City, or District
    election_scope: ElectionScope,
    chamber: Option<Chamber>,
    state: Option<State>,
    county: Option<String>,
    municipality: Option<String>,
    term_length: Option<i32>,
    seat: Option<String>,
    /// Used to determine order of offices in a list
    priority: Option<i32>,
}

//    Office subtitle rules
//    if (election_scope == state)
//      subtitle = state (full, e.g. "Minnesota")
//    if (political_scope == federal && election_scope == district)
//      subtitle = state abbv + " - District " + district (e.g. "MN - District 6")
//    if (political_scope == state && election_scope == district && district_type == state_house)
//      subtitle = state abbv + " - House District " + district (e.g. "MN - House District 3B")
//    if (political_scope == state && election_scope == district && district_type == state_senate)
//      subtitle = state abbv + " - Senate District " + district (e.g. "MN - Senate District 30")
//    if (political_scope == local && election_scope == city)
//      subtitle = municipality + ", " + state abbv (e.g. "St. Louis, MN")
//    if (political_scope == local && election_scope == district && district_type == city)
//      subtitle = municipality + ", " + state abbv + " - " + district (e.g. "St. Louis, MN - Ward 3")

fn compute_office_subtitle(office: &Office, use_short: bool) -> Option<String> {
    match (
        office.election_scope,
        office.political_scope,
        office.district_type,
    ) {
        (ElectionScope::National, _, _) => None,
        (ElectionScope::State, _, _) => office
            .state
            .as_ref()
            .map(|state| state.full_state().to_string()),
        (ElectionScope::District, PoliticalScope::Federal, Some(District::UsCongressional)) => {
            if let (Some(state), Some(district)) = (&office.state, &office.district) {
                Some(format!("{} - District {}", state, district))
            } else {
                None
            }
        }
        (ElectionScope::District, PoliticalScope::State, Some(District::StateHouse)) => {
            if let (Some(state), Some(district)) = (&office.state, &office.district) {
                Some(format!(
                    "{} - {} {}",
                    state,
                    if use_short { "HD" } else { "House District" },
                    district
                ))
            } else {
                None
            }
        }
        (ElectionScope::District, PoliticalScope::State, Some(District::StateSenate)) => {
            if let (Some(state), Some(district)) = (&office.state, &office.district) {
                Some(format!(
                    "{} - {} {}",
                    state,
                    if use_short { "SD" } else { "Senate District" },
                    district
                ))
            } else {
                None
            }
        }
        (ElectionScope::District, PoliticalScope::Local, Some(District::County)) => {
            if let (Some(county), Some(state), Some(district)) =
                (&office.county, &office.state, &office.district)
            {
                Some(format!(
                    "{} County, {} - District {}",
                    county, state, district
                ))
            } else {
                None
            }
        }
        (ElectionScope::District, PoliticalScope::Local, Some(District::City)) => {
            if let (Some(muni), Some(state), Some(district)) =
                (&office.municipality, &office.state, &office.district)
            {
                Some(format!("{}, {} - District {}", muni, state, district))
            } else {
                None
            }
        }
        (ElectionScope::District, PoliticalScope::Local, Some(District::School)) => {
            if let (Some(school_district), Some(state), Some(district)) =
                (&office.school_district, &office.state, &office.district)
            {
                Some(format!(
                    "{} - {} - District {}",
                    state, school_district, district
                ))
            } else if let (Some(school_district), Some(state)) =
                (&office.school_district, &office.state)
            {
                Some(format!("{} - {}", state, school_district))
            } else {
                None
            }
        }
        (ElectionScope::County, _, _) => office
            .county
            .as_ref()
            .map(|county| format!("{} County", county)),
        (ElectionScope::City, PoliticalScope::Local, _) => office
            .municipality
            .as_ref()
            .map(|muni| format!("{}, {}", muni, office.state.map(|s| s.to_string()).unwrap())),
        _ => None,
    }
}

#[ComplexObject]
impl OfficeResult {
    async fn incumbents(&self, ctx: &Context<'_>) -> Result<Vec<PoliticianResult>> {
        let politicians = ctx
            .data::<ApiContext>()?
            .loaders
            .politician_loader
            .load_many(vec![OfficeId(uuid::Uuid::parse_str(&self.id).unwrap())])
            .await?;
        let politician_results = politicians
            .values()
            .cloned()
            .collect::<Vec<Politician>>()
            .into_iter()
            .map(PoliticianResult::from)
            .collect();

        Ok(politician_results)
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
        Some("AL - Senate District 1".to_string())
    );
    assert_eq!(
        compute_office_subtitle(&office, true),
        Some("AL - SD 1".to_string())
    );

    let office = Office {
        id: uuid::Uuid::new_v4(),
        slug: "test-state-senator".to_string(),
        title: "State Senator".to_string(),
        subtitle: None,
        subtitle_short: None,
        name: None,
        office_type: None,
        district: Some("Ward 3".to_string()),
        district_type: Some(District::City),
        hospital_district: None,
        school_district: None,
        chamber: Some(Chamber::Senate),
        political_scope: PoliticalScope::Local,
        election_scope: ElectionScope::District,
        state: Some(State::AL),
        county: None,
        municipality: Some("Buckwild City".to_string()),
        term_length: None,
        seat: None,
        ..Default::default()
    };

    assert_eq!(
        compute_office_subtitle(&office, false),
        Some("Buckwild City, AL - Ward 3".to_string())
    )
}
