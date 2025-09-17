use std::error::Error;
use sqlx::PgPool;
use db::models::*;

pub struct CandidateFiling {
    pub office_title: String,
    pub office_id: String,
    pub candidate_name: String,
    pub party_abbreviation: String,
    pub campaign_phone: Option<String>,
    pub campaign_email: Option<String>,
    pub campaign_website: Option<String>,
    pub county_id: String,
    pub county_name: Option<String>,
    pub residence_street_address: Option<String>,
    pub residence_city: Option<String>,
    pub residence_state: Option<String>,
    pub residence_zip: Option<String>,
    pub campaign_address: Option<String>,
    pub campaign_city: Option<String>,
    pub campaign_state: Option<String>,
    pub campaign_zip: Option<String>,
}

pub async fn process_mn_candidate_filings(
    pool: &PgPool,
    source_table: &str,
    race_type: &str,
) -> Result<(), Box<dyn Error>> {
    // 1. Get raw filings from source table
    let filings = sqlx::query_as!(
        CandidateFiling,
        r#"
        SELECT 
            office_title,
            office_id,
            candidate_name,
            party_abbreviation,
            campaign_phone,
            campaign_email,
            campaign_website,
            county_id,
            vd.countyname as county_name,
            residence_street_address,
            residence_city,
            residence_state,
            residence_zip,
            campaign_address,
            campaign_city,
            campaign_state,
            campaign_zip
        FROM p6t_state_mn.mn_candidate_filings_local_primaries_2025 raw
        LEFT JOIN p6t_state_mn.bdry_votingdistricts vd
            ON REGEXP_REPLACE(raw.county_id, '^0+', '') = vd.countycode
        WHERE office_title != 'U.S. President & Vice President'
        "#
    )
    .fetch_all(pool)
    .await?;

    // 2. Process each filing
    for filing in filings {
        // Process office data
        let office = process_office(&filing)?;
        
        // Process politician data
        let politician = process_politician(&filing)?;
        
        // Process race data
        let race = process_race(&filing, &office, race_type)?;
        
        // Process race candidate relationship
        process_race_candidate(&filing, &race, &politician)?;
    }

    Ok(())
}

fn process_office(filing: &CandidateFiling) -> Result<Office, Box<dyn Error>> {
    // Extract office attributes
    let name = extractors::mn::office::extract_office_name(&filing.office_title);
    let title = extractors::mn::office::extract_office_title(&filing.office_title)
        .ok_or("Failed to extract office title")?;
    let chamber = extractors::mn::office::extract_office_chamber(&filing.office_title);
    let district_type = extractors::mn::office::extract_office_district_type(
        &filing.office_title,
        filing.county_id.parse::<i32>().ok(),
    );
    let political_scope = extractors::mn::office::extract_office_political_scope(&filing.office_title)
        .ok_or("Failed to extract political scope")?;
    let election_scope = extractors::mn::office::extract_office_election_scope(
        &filing.office_title,
        filing.county_id.parse::<i32>().ok(),
    ).ok_or("Failed to extract election scope")?;

    // Extract district and seat
    let district = extractors::mn::office::extract_office_district(&filing.office_title);
    let seat = extractors::mn::office::extract_office_seat(&filing.office_title);

    // Extract school and hospital districts
    let school_district = extractors::mn::office::extract_school_district(&filing.office_title);
    let hospital_district = extractors::mn::office::extract_hospital_district(&filing.office_title);

    // Extract municipality
    let municipality = extractors::mn::office::extract_municipality(
        &filing.office_title,
        &election_scope,
        &district_type,
    );

    // Generate slug and subtitle
    let slug = generators::mn::office::OfficeSlugGenerator {
        state: &State::MN,
        name: name.as_deref(),
        county: filing.county_name.as_deref(),
        district: district.as_deref(),
        seat: seat.as_deref(),
        school_district: school_district.as_deref(),
        hospital_district: hospital_district.as_deref(),
        municipality: municipality.as_deref(),
        election_scope: Some(&election_scope),
        district_type: district_type.as_ref(),
    }.generate();

    let (subtitle, subtitle_short) = generators::mn::office::OfficeSubtitleGenerator {
        state: &State::MN,
        county: filing.county_name.as_deref(),
        district: district.as_deref(),
        seat: seat.as_deref(),
    }.generate();

    // Create office record
    Ok(Office {
        slug,
        name,
        title,
        subtitle,
        subtitle_short,
        chamber,
        district_type,
        political_scope,
        election_scope,
        state: State::MN,
        county: filing.county_name.clone(),
        district,
        seat,
        school_district,
        hospital_district,
        municipality,
        ..Default::default()
        //priority
        //state id
        //ref key?
    })
}

fn process_politician(filing: &CandidateFiling) -> Result<Politician, Box<dyn Error>> {
    // Generate politician slug
    let slug = generators::mn::politician::PoliticianSlugGenerator::new(&filing.candidate_name).generate();
    
    // Generate ref key
    let ref_key = generators::mn::politician::PoliticianRefKeyGenerator::new("MN-SOS", &filing.candidate_name).generate();
    
    // Process party
    let party = if let Some(party_name) = extractors::mn::party::extract_party_name(&filing.party_abbreviation) {
        let party_slug = generators::mn::party::PartySlugGenerator::new(&party_name).generate();
        Some(Party {
            name: party_name,
            slug: party_slug,
            ..Default::default()
        })
    } else {
        None
    };

    // Create politician record
    Ok(Politician {
        slug,
        ref_key,
        full_name: filing.candidate_name.clone(),
        party_id: party.map(|p| p.id),
        campaign_website_url: filing.campaign_website.clone(),
        email: filing.campaign_email.clone(),
        phone: filing.campaign_phone.clone(),
        ..Default::default()
    })
}

fn process_race(
    filing: &CandidateFiling,
    office: &Office,
    race_type: &str,
) -> Result<Race, Box<dyn Error>> {
    // Generate race title and slug
    let (title, slug) = generators::mn::race::RaceTitleGenerator::from_source(
        &RaceType::from_str(race_type)?,
        &Election {
            slug: format!("{}-election-2024", race_type),
            ..Default::default()
        },
        office,
    ).generate();

    // Create race record
    Ok(Race {
        title,
        slug,
        office_id: office.id,
        state: State::MN,
        race_type: RaceType::from_str(race_type)?,
        vote_type: VoteType::Plurality,
        ..Default::default()
    })
}

fn process_race_candidate(
    filing: &CandidateFiling,
    race: &Race,
    politician: &Politician,
) -> Result<RaceCandidate, Box<dyn Error>> {
    Ok(RaceCandidate {
        race_id: race.id,
        candidate_id: politician.id,
    })
} 