use db::{
    models::enums::{PoliticalParty, PoliticalScope, RaceType, State},
    District, ElectionScope, UpsertPoliticianInput,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use slugify::slugify;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io, process,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct CandidateFiling {
    office_code: Option<i64>,
    office_id: i64,
    full_name: String,
    office_title: String,
    county_id: i32,
    mcd_fips_code: Option<i64>,
    school_district_number: Option<i64>,
    party: String,
    residence_street: Option<String>,
    residence_city: Option<String>,
    residence_state: Option<String>,
    residence_zip: Option<String>,
    campaign_street: Option<String>,
    campaign_city: Option<String>,
    campaign_zip: Option<String>,
    campaign_phone: Option<String>,
    campaign_website: Option<String>,
    campaign_email: Option<String>,
}

// Regex for capturing multiple words within parentheses

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
struct FilingOffice {
    id: i64,
    title: String,
    district: Option<String>,
    county_name: Option<String>,
    election_scope: Option<ElectionScope>,
}

struct ParsedTitle {
    title: String,
    district: Option<String>,
    municipality: Option<String>,
    seat: Option<String>,
}

fn parse_title_and_district(title: &str) -> ParsedTitle {
    let re = regex::Regex::new(r#"(\d+)"#).unwrap();
    let district = if title.contains("District Court") {
        let title = title
            .split("District Court")
            .nth(0)
            .unwrap_or_default()
            .split("-")
            .nth(1)
            .unwrap_or_default()
            .trim();
        let caps = re.captures(title).unwrap();
        let district = caps[1].to_string();
        district
    } else {
        title
            .split("District")
            .nth(1)
            .unwrap_or_default()
            .trim()
            .to_string()
    };

    let title_clone = title.clone();

    let seat = match title_clone {
        t if t.contains("Court of Appeals") => {
            let seat = title
                .split("Court of Appeals")
                .nth(1)
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(seat)
        }
        t if t.contains("Supreme Court") => {
            let seat = title
                .split("Supreme Court")
                .nth(1)
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(seat)
        }
        t if t.contains("District Court") => {
            let seat = title
                .split("District Court")
                .nth(1)
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(seat)
        }
        _ => None,
    };
    let district = match district {
        d if d.is_empty() => None,
        d => Some(d),
    };

    // First words within parens that don't include "ISD"
    let muni_re = regex::Regex::new(r"\(([^ISD)]+)\)").unwrap();
    let municipality = muni_re.captures(&title).map(|c| c[1].to_string());

    ParsedTitle {
        title: title.to_string(),
        district,
        municipality,
        seat,
    }
}

// Hashmap with Minnesota counties by code
fn minnesota_counties() -> HashMap<i32, &'static str> {
    let counties: HashMap<i32, &str> = [
        (1, "Aitkin"),
        (2, "Anoka"),
        (3, "Becker"),
        (4, "Beltrami"),
        (5, "Benton"),
        (6, "Big Stone"),
        (7, "Blue Earth"),
        (8, "Brown"),
        (9, "Carlton"),
        (10, "Carver"),
        (11, "Cass"),
        (12, "Chippewa"),
        (13, "Chisago"),
        (14, "Clay"),
        (15, "Clearwater"),
        (16, "Cook"),
        (17, "Cottonwood"),
        (18, "Crow Wing"),
        (19, "Dakota"),
        (20, "Dodge"),
        (21, "Douglas"),
        (22, "Faribault"),
        (23, "Fillmore"),
        (24, "Freeborn"),
        (25, "Goodhue"),
        (26, "Grant"),
        (27, "Hennepin"),
        (28, "Houston"),
        (29, "Hubbard"),
        (30, "Isanti"),
        (31, "Itasca"),
        (32, "Jackson"),
        (33, "Kanabec"),
        (34, "Kandiyohi"),
        (35, "Kittson"),
        (36, "Koochiching"),
        (37, "Lac qui Parle"),
        (38, "Lake"),
        (39, "Lake of the Woods"),
        (40, "Le Sueur"),
        (41, "Lincoln"),
        (42, "Lyon"),
        (43, "McLeod"),
        (44, "Mahnomen"),
        (45, "Marshall"),
        (46, "Martin"),
        (47, "Meeker"),
        (48, "Mille Lacs"),
        (49, "Morrison"),
        (50, "Mower"),
        (51, "Murray"),
        (52, "Nicollet"),
        (53, "Nobles"),
        (54, "Norman"),
        (55, "Olmsted"),
        (56, "Otter Tail"),
        (57, "Pennington"),
        (58, "Pine"),
        (59, "Pipestone"),
        (60, "Polk"),
        (61, "Pope"),
        (62, "Ramsey"),
        (63, "Red Lake"),
        (64, "Redwood"),
        (65, "Renville"),
        (66, "Rice"),
        (67, "Rock"),
        (68, "Roseau"),
        (69, "St. Louis"),
        (70, "Scott"),
        (71, "Sherburne"),
        (72, "Sibley"),
        (73, "Stearns"),
        (74, "Steele"),
        (75, "Stevens"),
        (76, "Swift"),
        (77, "Todd"),
        (78, "Traverse"),
        (79, "Wabasha"),
        (80, "Wadena"),
        (81, "Waseca"),
        (82, "Washington"),
        (83, "Watonwan"),
        (84, "Wilkin"),
        (85, "Winona"),
        (86, "Wright"),
        (87, "Yellow Medicine"),
    ]
    .iter()
    .cloned()
    .collect();

    counties
}

fn create_filing_office_from_csv_row(candidate_filing: &CandidateFiling) -> FilingOffice {
    let ParsedTitle { district, .. } = parse_title_and_district(&candidate_filing.office_title);

    let election_scope = match candidate_filing.county_id {
        88 => ElectionScope::State,
        _ => ElectionScope::County,
    };

    FilingOffice {
        id: candidate_filing.office_id,
        title: candidate_filing.clone().office_title,
        district,
        county_name: minnesota_counties()
            .get(&candidate_filing.county_id)
            .cloned()
            .map(|a| a.to_string()),
        election_scope: Some(election_scope),
    }
}

#[derive(Debug, Clone)]
struct ParsedName {
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    suffix: Option<String>,
    preferred_name: Option<String>,
}

fn parse_human_name(name: &str) -> ParsedName {
    let parsed_name = human_name::Name::parse(&name.replace("St.", "St"))
        .expect(&format!("Failed to parse name: {}", name));
    let double_quotes_re = regex::Regex::new(r#""(.*?)""#).unwrap();
    let mut preferred_name = double_quotes_re.captures(name).map(|c| c[1].to_string());

    if preferred_name.is_none() {
        let re = regex::Regex::new(r#"\((.*?)\)"#).unwrap();
        preferred_name = re.captures(name).map(|c| c[1].to_string());
    }

    let middle_name = match parsed_name.middle_name() {
        Some(m) => Some(m.to_string()),
        None => match parsed_name.middle_initials() {
            Some(m) => Some(m.to_string()),
            None => None,
        },
    };

    ParsedName {
        first_name: parsed_name.given_name().map_or("", |a| a).to_string(),
        middle_name,
        last_name: parsed_name.surname().to_string(),
        suffix: parsed_name.suffix().map(|a| a.to_string()),
        preferred_name,
    }
}

async fn seed_minnesota_sos_data() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let mut rdr = csv::Reader::from_reader(io::stdin());

    // Loop through all candidate filings and create a hashset of unique offices
    let mut offices = HashSet::new();
    let mut filings = HashSet::new();
    for result in rdr.deserialize() {
        let candidate_filing: CandidateFiling = result?;
        filings.insert(candidate_filing.clone());
        // Insert each unique office into the HashSet
        offices.insert(create_filing_office_from_csv_row(&candidate_filing));
    }

    // Create a new office record and associated general race for each office in HashSet
    // Also create new HashMap to associate new races with candidate filings in memory
    let mut races = HashMap::new();

    println!("Creating office records");
    for office in offices.clone() {
        let office_clone = office.clone();
        let ParsedTitle {
            title,
            seat,
            municipality,
            ..
        } = parse_title_and_district(&office.title);

        let mut rng = rand::thread_rng();
        let rnd_int: i32 = rng.gen();

        // Slug format: [title]-[municipality]-[state]-[district]
        let slug = slugify!(&format!(
            "{} {} {} {}",
            "mn".to_string(),
            office.clone().county_name.unwrap_or("".to_string()),
            &office.title,
            rnd_int
        ));

        let district_type = match title.clone() {
            t if t.contains("County Commissioner") => Some(District::County),
            t if t.contains("County Park Commissioner") => Some(District::County),
            t if t.contains("Soil and Water Supervisor") => Some(District::County),
            t if t.contains("Hospital") => Some(District::City),
            t if t.contains("Sanitary") => Some(District::City),
            t if t.contains("School Board") => Some(District::School),
            t if t.contains("State Senator") => Some(District::StateSenate),
            t if t.contains("State Representative") => Some(District::StateHouse),
            t if t.contains("U.S. House") => Some(District::UsCongressional),
            _ => None,
        };

        let (political_scope, election_scope) = match title.clone() {
            t if t.contains("U.S. Representative") => {
                (PoliticalScope::Federal, ElectionScope::National)
            }
            t if t.contains("State Senator") | t.contains("State Representative") => {
                (PoliticalScope::State, ElectionScope::District)
            }
            t if t.contains("Supreme Court") => (PoliticalScope::State, ElectionScope::State),
            t if t.contains("Court of Appeals") => (PoliticalScope::State, ElectionScope::State),
            t if t.contains("County Attorney")
                || t.contains("County Sheriff")
                || t.contains("County Recorder") =>
            {
                (PoliticalScope::State, ElectionScope::State)
            }
            t if t.contains("County Commissioner")
                || t.contains("Hospital District")
                || t.contains("Sanitary District")
                || t.contains("School Board") =>
            {
                (PoliticalScope::Local, ElectionScope::District)
            }
            t if t.contains("County Auditor")
                || t.contains("County Treasurer")
                || t.contains("County Auditor / Treasurer") =>
            {
                (PoliticalScope::Local, ElectionScope::County)
            }
            t if t.contains("Attorney General")
                || t.contains("State Auditor")
                || t.contains("Secretary of State")
                || t.contains("Governor") =>
            {
                (PoliticalScope::State, ElectionScope::State)
            }
            t if t.contains("County Surveyor") || t.contains("County  Coroner") => {
                (PoliticalScope::Local, ElectionScope::County)
            }
            t if t.contains("County Park Commissioner")
                || t.contains("Soil and Water Supervisor") =>
            {
                (PoliticalScope::Local, ElectionScope::District)
            }
            _ => (PoliticalScope::Local, ElectionScope::City),
        };

        let municipality = if municipality.is_some() {
            municipality
        } else if office.county_name.is_some() {
            office.county_name
        } else {
            None
        };

        let office_input = db::UpsertOfficeInput {
            slug: Some(slug.clone()),
            title: Some(title.clone()),
            district: office.district,
            district_type,
            state: Some(State::MN),
            municipality,
            seat,
            political_scope: Some(political_scope),
            election_scope: Some(election_scope),
            ..Default::default()
        };

        let created_office = db::Office::upsert(&pool.connection, &office_input)
            .await
            .expect(&format!("Failed creating office: {}", slug));

        let race_input = db::CreateRaceInput {
            slug: Some(slug),
            title,
            office_id: created_office.id,
            race_type: RaceType::General,
            party: None,
            state: Some(State::MN),
            description: None,
            ballotpedia_link: None,
            early_voting_begins_date: None,
            official_website: None,
            election_id: None,
            winner_id: None,
        };

        let created_race = db::Race::create(&pool.connection, &race_input)
            .await
            .expect("Failed created race");

        races.insert(office_clone, created_race.id);
    }
    tracing::info!(
        "Created {} office records and {} race records",
        offices.len(),
        races.len()
    );

    println!("Creating politician records");
    // Loop through candidate filings and create new politician records and new race_candidates records for each
    for filing in filings.clone() {
        let filing_clone = filing.clone();
        let slug = slugify!(&filing.full_name).to_string();
        let party = match filing.party.as_ref() {
            "DFL" => PoliticalParty::DemocraticFarmerLabor,
            "LMN" => PoliticalParty::LegalMarijuanaNow,
            "GLC" => PoliticalParty::GrassrootsLegalizeCannabis,
            "R" => PoliticalParty::Republican,
            _ => PoliticalParty::Unaffiliated,
        };

        let ParsedName {
            first_name,
            middle_name,
            last_name,
            suffix,
            preferred_name,
        } = parse_human_name(&filing.full_name);

        // A regex that captures any string between parentheses or quotes
        let new_politician_input = UpsertPoliticianInput {
            slug: Some(slug),
            first_name: Some(first_name),
            middle_name,
            last_name: Some(last_name),
            suffix,
            preferred_name,
            campaign_website_url: filing.campaign_website,
            email: filing.campaign_email,
            phone: filing.campaign_phone,
            party: Some(party),
            home_state: Some(State::MN),
            ..Default::default()
        };
        let created_politician = db::Politician::upsert(&pool.connection, &new_politician_input)
            .await
            .expect(
                format!(
                    "Something went wrong creating politician: {}",
                    &filing.full_name
                )
                .as_str(),
            );

        let filing_office = create_filing_office_from_csv_row(&filing_clone);
        let race_id = races.get(&filing_office);

        let _created_race_candidate = sqlx::query!(
            r#"
                    INSERT INTO race_candidates (race_id, candidate_id) VALUES ($1, $2) RETURNING *
                "#,
            race_id,
            created_politician.id
        )
        .fetch_one(&pool.connection)
        .await
        .expect("Failed to create race_candidate record");
    }
    tracing::info!(
        "Created {} politician and race_candidate_records",
        filings.len()
    );

    Ok(())
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    if let Err(err) = seed_minnesota_sos_data().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
