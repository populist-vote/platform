use db::{
    models::enums::{PoliticalParty, PoliticalScope, RaceType, State},
    CreatePoliticianInput, ElectionScope,
};
use human_name::Name;
use serde::{Deserialize, Serialize};
use slugify::slugify;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io, process,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct CandidateFiling {
    office_id: i32,
    full_name: String,
    office_title: String,
    county_id: i32,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
struct FilingOffice {
    id: i32,
    title: String,
    district: Option<String>,
    county_name: Option<String>,
    election_scope: Option<ElectionScope>,
}

fn parse_district_from_office_title(title: &str) -> Option<String> {
    let district = if title.contains("District Court") {
        title
            .split("District Court")
            .nth(0)
            .unwrap_or_default()
            .split("-")
            .nth(1)
            .unwrap_or_default()
            .trim()
            .to_string()
            .chars()
            .find(|a| a.is_digit(10))
            .unwrap()
            .to_string()
    } else {
        title
            .split("District")
            .nth(1)
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let district = match district {
        d if d.is_empty() => None,
        d => Some(d),
    };
    district
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
    let district = parse_district_from_office_title(&candidate_filing.office_title);
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
    tracing::info!("Total offices parsed = {:?}", offices.len());

    // Create a new office record and associated general race for each office in HashSet
    // Also create new HashMap to associate new races with candidate filings in memory
    let mut races = HashMap::new();
    for office in offices.clone() {
        let office_clone = office.clone();
        let slug = slugify!(&format!(
            "{} {}",
            &office.clone().county_name.unwrap_or("".to_string()),
            &office.title
        ));
        let title = format!(
            "{} {}",
            &office.clone().county_name.unwrap_or("".to_string()),
            &office.title
        );
        let political_scope = match title.clone() {
            t if t.contains("County") => PoliticalScope::Local,
            t if t.contains("U.S. Representative") => PoliticalScope::Federal,
            _ => PoliticalScope::State,
        };

        let new_office_input = db::CreateOfficeInput {
            slug: Some(slug.clone()),
            title: title.clone(),
            district: office.district,
            state: Some(State::MN),
            municipality: office.county_name,
            political_scope,
            ..Default::default()
        };

        let created_office = db::Office::create(&pool.connection, &new_office_input)
            .await
            .expect(&format!("Failed creating office: {}", slug));

        let race_input = db::CreateRaceInput {
            slug: Some(slugify!(&title)),
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

    // Loop through candidate filings and create new politician records and new race_candidates records for each
    for filing in filings.clone() {
        let filing_clone = filing.clone();
        let slug = slugify!(&filing.full_name).to_string();
        let name = Name::parse(&filing.full_name)
            .expect(&format!("Failed to parse name: {}", &filing.full_name));
        let party = match filing.party.as_ref() {
            "DFL" => PoliticalParty::Democratic,
            "R" => PoliticalParty::Republican,
            _ => PoliticalParty::Unaffiliated,
        };

        let new_politician_input = CreatePoliticianInput {
            slug: Some(slug),
            first_name: name.given_name().unwrap_or_default().to_string(),
            middle_name: name.middle_name().map(|s| s.to_string()),
            last_name: name.surname().to_string(),
            suffix: name.suffix().map(|s| s.to_string()),
            campaign_website_url: filing.campaign_website,
            email: filing.campaign_email,
            party: Some(party),
            home_state: Some(State::MN),
            ..Default::default()
        };
        let created_politician = db::Politician::create(&pool.connection, &new_politician_input)
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
