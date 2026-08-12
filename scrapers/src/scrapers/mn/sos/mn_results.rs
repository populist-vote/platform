use crate::generators::politician::PoliticianRefKeyGenerator;
use csv::ReaderBuilder;
use encoding_rs::WINDOWS_1252;
use reqwest::Client;
use sqlx::PgPool;
use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::fs::File;
use uuid::Uuid;

/// Election whose slug is included in race_candidate ref_keys when matching SOS results.
/// Must match the election used when processing MN candidate filings.
const ELECTION_ID: &str = "5fa881d7-f8f3-4b90-9063-45236c85c77a";

static HEADER_NAMES: [&str; 16] = [
    "State",
    "County ID",
    "Precinct name",
    "Office ID",
    "Office name",
    "District",
    "Candidate order code",
    "Candidate name",
    "Suffix",
    "Incumbent code",
    "Party abbreviation",
    "Number of precincts reporting",
    "Total number of precincts voting for the office",
    "Votes for candidate",
    "Percentage of votes for candidate out of total votes for Office",
    "Total number of votes for Office in area",
];

static PRECINCT_STATS_HEADER_NAMES: [&str; 12] = [
    "State",
    "County ID",
    "Precinct ID",
    "Precinct Name",
    "Has Reported Statistics",     // (1 = yes, 0 = no)
    "Number of Voters Registered", // as of 7:00 a.m. Election Day
    "Number of Voters that Registered on Election Day",
    "Number of Signatures on the Polling Place Roster",
    "Number of Regular Military and Overseas Absentee Ballots",
    "Number of Federal Only Absentee Ballots",
    "Number of President Only Absentee Ballots",
    "Total Number Voted",
];

#[derive(Debug, sqlx::FromRow)]
struct ResultIdentity {
    office_name: String,
    candidate_name: String,
}

#[derive(Debug)]
struct RefKeyCandidate {
    office_name: String,
    candidate_name: String,
    ref_key: String,
    priority: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct UpdateSummary {
    result_rows: i64,
    matched_rows: i64,
    unmatched_candidates: Option<String>,
}

pub async fn fetch_results() -> Result<(), Box<dyn Error>> {
    let mut results_file_paths: HashMap<&str, &str> = HashMap::new();

    results_file_paths.insert(
        "U.S. Senator Statewide",
        "https://electionresultsfiles.sos.mn.gov/20260811/ussenate.txt",
    );
    results_file_paths.insert(
        "U.S. Representative by District",
        "https://electionresultsfiles.sos.mn.gov/20260811/ushouse.txt",
    );
    results_file_paths.insert(
        "Governor Statewide",
        "https://electionresultsfiles.sos.mn.gov/20260811/Governor.txt",
    );
    results_file_paths.insert(
        "Secretary of State Statewide",
        "https://electionresultsfiles.sos.mn.gov/20260811/secofstate.txt",
    );
    results_file_paths.insert(
        "Attorney General Statewide",
        "https://electionresultsfiles.sos.mn.gov/20260811/attorneygen.txt",
    );
    results_file_paths.insert(
        "State Auditor Statewide",
        "https://electionresultsfiles.sos.mn.gov/20260811/auditor.txt",
    );
    // results_file_paths.insert(
    //     "Supreme Court and Courts of Appeals",
    //     "https://electionresultsfiles.sos.state.mn.us/20241105/judicial.txt",
    // );
    results_file_paths.insert(
        "State Senator by District",
        "https://electionresultsfiles.sos.mn.gov/20260811/stsenate.txt",
    );
    results_file_paths.insert(
        "State Representative by District",
        "https://electionresultsfiles.sos.mn.gov/20260811/LegislativeByDistrict.txt",
    );
    results_file_paths.insert(
        "District Court Races",
        "https://electionresultsfiles.sos.mn.gov/20260811/judicialdst.txt",
    );
    results_file_paths.insert(
        "County Races",
        "https://electionresultsfiles.sos.mn.gov/20260811/cntyRaces.txt",
    );
    results_file_paths.insert(
        "Municipal Races",
        "https://electionresultsfiles.sos.mn.gov/20260811/local.txt",
    );
    results_file_paths.insert(
        "School Board Races",
        "https://electionresultsfiles.sos.mn.gov/20260811/sdrace.txt",
    );

    let client = Client::new();
    let mut table_names = Vec::new();
    for (name, url) in results_file_paths {
        let response = client.get(url).send().await?.error_for_status()?;
        // MN SoS result files are Windows-1252, even though the response does not
        // consistently declare that charset. Reqwest's default UTF-8 decoding
        // replaces accented characters and prevents their ref_keys from matching.
        let response = decode_results_text(&response.bytes().await?);
        let data = convert_text_to_csv(name, &response)?;
        let csv_data_as_string = String::from_utf8(data.clone())?;
        let table_name = format!(
            "p6t_state_mn.results_{}_{}",
            url.split('/')
                .find(|segment| { segment.len() == 8 && segment.chars().all(|c| c.is_numeric()) })
                .unwrap_or_else(|| {
                    tracing::warn!("No valid date segment found in URL: {}", url);
                    "unknown"
                }),
            name.replace(['.', ','], "")
                .replace(' ', "_")
                .to_lowercase()
        );
        table_names.push(table_name.clone());
        let copy_query = format!("COPY {} FROM STDIN WITH CSV HEADER;", table_name);
        let pool = db::pool().await;
        sqlx::query(format!(r#"DROP TABLE IF EXISTS {} CASCADE;"#, table_name).as_str())
            .execute(&pool.connection)
            .await?;

        let create_table_query = get_create_table_query(name, table_name.as_str());

        sqlx::query(&create_table_query)
            .execute(&pool.connection)
            .await?;
        let mut tx = pool.connection.acquire().await?;
        let mut tx_copy = tx.copy_in_raw(&copy_query).await?;
        tx_copy.send(csv_data_as_string.as_bytes()).await?;
        tx_copy.finish().await?;
        // TODO: Refactor this scraper to fit the Scraper interface with run_local fn and remove below line
        _write_to_csv_file(name, &data)?;
    }
    update_public_schema_with_results(table_names).await?;

    Ok(())
}

fn decode_results_text(bytes: &[u8]) -> String {
    let (text, _, _) = WINDOWS_1252.decode(bytes);
    text.into_owned()
}

fn get_create_table_query(name: &str, table_name: &str) -> String {
    if name == "Precinct Reporting Statistics" {
        return format!(
            "CREATE TABLE {} (
            {}
        );",
            table_name,
            PRECINCT_STATS_HEADER_NAMES
                .iter()
                .map(|&name| format!("{} text", name.replace(' ', "_").to_lowercase()))
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
    format!(
        "CREATE TABLE {} (
            {}
        );",
        table_name,
        HEADER_NAMES
            .iter()
            .map(|&name| format!("{} text", name.replace(' ', "_").to_lowercase()))
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn convert_text_to_csv(name: &str, text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_reader(text.as_bytes());
    let mut csv_string = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut csv_string);

        // Write the headers from the above struct
        if name == "Precinct Reporting Statistics" {
            wtr.write_record(PRECINCT_STATS_HEADER_NAMES)?;
        } else {
            wtr.write_record(HEADER_NAMES)?;
        }
        for result in reader.records() {
            // test that record is valid
            let record = match result {
                Ok(record) => record,
                Err(e) => {
                    println!("Error reading record: {:?}", e);
                    continue;
                }
            };
            // test to ensure record has 16 parts
            if record.len() != 16 {
                tracing::error!("Record has {} parts, expected 16: {:?}", record.len(), name);
                continue;
            }
            wtr.write_record(&record)
                .unwrap_or_else(|_| panic!("Error writing record: {:?}", record))
        }
        wtr.flush()?;
    }

    Ok(csv_string)
}

fn candidate_name_without_parentheticals(candidate_name: &str) -> Option<String> {
    let mut depth = 0_u32;
    let mut changed = false;
    let mut output = String::with_capacity(candidate_name.len());

    for character in candidate_name.chars() {
        match character {
            '(' => {
                depth += 1;
                changed = true;
            }
            ')' if depth > 0 => depth -= 1,
            _ if depth == 0 => output.push(character),
            _ => {}
        }
    }

    let output = output.split_whitespace().collect::<Vec<_>>().join(" ");
    (changed && !output.is_empty()).then_some(output)
}

fn result_ref_key_candidates(
    election_slug: &str,
    office_name: &str,
    candidate_name: &str,
) -> Vec<String> {
    let generate = |name: &str| {
        PoliticianRefKeyGenerator::new("mn-sos", election_slug, office_name, Some(name)).generate()
    };

    let mut name_candidates = vec![candidate_name.to_string()];

    // The results feed sometimes includes a parenthesized preferred name that
    // is absent from the certified candidate filing (for example, Raafat).
    if let Some(candidate_name) = candidate_name_without_parentheticals(candidate_name) {
        if !name_candidates.contains(&candidate_name) {
            name_candidates.push(candidate_name);
        }
    }

    // Candidate-name processing may normalize dotted suffixes such as J.R. to
    // Jr. Generate both forms so result ingestion remains compatible with data
    // processed before and after that normalization.
    for candidate_name in name_candidates.clone() {
        let without_periods = candidate_name.replace('.', "");
        if without_periods != candidate_name && !name_candidates.contains(&without_periods) {
            name_candidates.push(without_periods);
        }
    }

    let mut ref_keys = Vec::new();
    for candidate_name in name_candidates {
        let ref_key = generate(&candidate_name);
        if !ref_keys.contains(&ref_key) {
            ref_keys.push(ref_key);
        }
    }

    ref_keys
}

fn normalized_result_identities(office_name: &str, candidate_name: &str) -> Vec<(String, String)> {
    if candidate_name.trim().eq_ignore_ascii_case("WRITE-IN") {
        return Vec::new();
    }

    if office_name.trim() != "Governor & Lt Governor" {
        return vec![(
            office_name.trim().to_string(),
            candidate_name.trim().to_string(),
        )];
    }

    let Some((governor, lieutenant_governor)) = candidate_name.split_once(" and ") else {
        return Vec::new();
    };

    vec![
        ("Governor".to_string(), governor.trim().to_string()),
        (
            "Lieutenant Governor".to_string(),
            lieutenant_governor.trim().to_string(),
        ),
    ]
}

async fn build_ref_key_candidates(
    pool: &PgPool,
    source_tables: &str,
    election_slug: &str,
) -> Result<Vec<RefKeyCandidate>, Box<dyn Error>> {
    let identities_query = format!(
        r#"
        SELECT DISTINCT office_name, candidate_name
        FROM ({}) AS source
        WHERE office_name IS NOT NULL AND candidate_name IS NOT NULL
        "#,
        source_tables
    );
    let source_identities = sqlx::query_as::<_, ResultIdentity>(&identities_query)
        .fetch_all(pool)
        .await?;

    let mut normalized_identities = BTreeSet::new();
    for identity in source_identities {
        normalized_identities.extend(normalized_result_identities(
            &identity.office_name,
            &identity.candidate_name,
        ));
    }

    let mut ref_keys = Vec::new();
    for (office_name, candidate_name) in normalized_identities {
        for (priority, ref_key) in
            result_ref_key_candidates(election_slug, &office_name, &candidate_name)
                .into_iter()
                .enumerate()
        {
            ref_keys.push(RefKeyCandidate {
                office_name: office_name.clone(),
                candidate_name: candidate_name.clone(),
                ref_key,
                priority: priority as i32,
            });
        }
    }

    Ok(ref_keys)
}

async fn update_public_schema_with_results(table_names: Vec<String>) -> Result<(), Box<dyn Error>> {
    let db_pool = db::pool().await;

    let election_id = Uuid::parse_str(ELECTION_ID)?;
    let election_slug: String =
        sqlx::query_scalar!(r#"SELECT slug FROM election WHERE id = $1"#, election_id)
            .fetch_optional(&db_pool.connection)
            .await?
            .ok_or_else(|| {
                std::io::Error::other(format!("No election found for id {}", ELECTION_ID))
            })?;
    println!(
        "Matching race_candidates with election slug '{}' (id {})",
        election_slug, ELECTION_ID
    );

    // Build the source CTE dynamically from the provided table names
    let source_tables = table_names
        .iter()
        .map(|table| format!("SELECT * FROM {}", table))
        .collect::<Vec<String>>()
        .join(" UNION ALL ");

    // Generate keys in Rust with the same implementation used by candidate
    // ingestion. Reimplementing slugify with PostgreSQL regexes caused drift for
    // accents, apostrophes, slashes, and parenthesized preferred names.
    let ref_key_candidates =
        build_ref_key_candidates(&db_pool.connection, &source_tables, &election_slug).await?;
    if ref_key_candidates.is_empty() {
        return Err(std::io::Error::other("No Minnesota result identities found").into());
    }

    let ref_key_values = ref_key_candidates
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let first = index * 4 + 1;
            format!(
                "(${}::text, ${}::text, ${}::text, ${}::integer)",
                first,
                first + 1,
                first + 2,
                first + 3
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        r#"
        WITH ref_key_candidates (office_name, candidate_name, ref_key, priority) AS (
            VALUES {}
        ),
        source AS (
            {}
        ),
        source_normalized AS (
            SELECT
                office_name,
                county_id,
                office_id,
                candidate_name,
                votes_for_candidate,
                total_number_of_votes_for_office_in_area,
                number_of_precincts_reporting,
                total_number_of_precincts_voting_for_the_office
            FROM
                source
            WHERE
                TRIM(office_name) IS DISTINCT FROM 'Governor & Lt Governor'
                AND UPPER(TRIM(COALESCE(candidate_name, ''))) <> 'WRITE-IN'
            UNION ALL
            SELECT
                'Governor' AS office_name,
                county_id,
                office_id,
                TRIM(SPLIT_PART(candidate_name, ' and ', 1)) AS candidate_name,
                votes_for_candidate,
                total_number_of_votes_for_office_in_area,
                number_of_precincts_reporting,
                total_number_of_precincts_voting_for_the_office
            FROM
                source
            WHERE
                TRIM(office_name) = 'Governor & Lt Governor'
                AND TRIM(SPLIT_PART(COALESCE(candidate_name, ''), ' and ', 1)) <> ''
            UNION ALL
            SELECT
                'Lieutenant Governor' AS office_name,
                county_id,
                office_id,
                TRIM(SPLIT_PART(candidate_name, ' and ', 2)) AS candidate_name,
                votes_for_candidate,
                total_number_of_votes_for_office_in_area,
                number_of_precincts_reporting,
                total_number_of_precincts_voting_for_the_office
            FROM
                source
            WHERE
                TRIM(office_name) = 'Governor & Lt Governor'
                AND POSITION(' and ' IN COALESCE(candidate_name, '')) > 0
                AND TRIM(SPLIT_PART(candidate_name, ' and ', 2)) <> ''
        ),
        results AS (
            SELECT DISTINCT ON (office_name, candidate_name)
                office_name,
                source.county_id,
                source.office_id,
                candidate_name,
                votes_for_candidate,
                total_number_of_votes_for_office_in_area,
                number_of_precincts_reporting,
                total_number_of_precincts_voting_for_the_office,
                matched.ref_key,
                matched.race_id,
                r.title AS race_title,
                r.vote_type AS vote_type,
                matched.race_candidate_votes,
                r.total_votes AS race_total_votes,
                CASE WHEN office_name ILIKE '%first choice%' THEN
                    votes_for_candidate::int
                ELSE
                    NULL
                END AS first_choice_votes,
                CASE WHEN office_name ILIKE '%first choice%' THEN
                    total_number_of_votes_for_office_in_area::int
                ELSE
                    NULL
                END AS total_first_choice_votes
            FROM
                source_normalized AS source
            LEFT JOIN LATERAL (
                SELECT
                    rc.ref_key,
                    race.id AS race_id,
                    rc.votes AS race_candidate_votes
                FROM
                    ref_key_candidates keys
                JOIN race_candidates rc ON rc.ref_key = keys.ref_key
                JOIN race ON race.id = rc.race_id
                    AND race.election_id = '{}'::uuid
                WHERE
                    keys.office_name = source.office_name
                    AND keys.candidate_name = source.candidate_name
                ORDER BY
                    keys.priority
                LIMIT 1
            ) matched ON TRUE
            LEFT JOIN race r ON r.id = matched.race_id
            ORDER BY
                office_name,
                candidate_name,
                CASE WHEN office_name LIKE '%First Choice%' THEN
                    1
                WHEN office_name LIKE '%Second Choice%' THEN
                    2
                WHEN office_name LIKE '%Third Choice%' THEN
                    3
                ELSE
                    4 -- You can add more conditions if needed
                END
        ),
        update_race_candidates AS (
            UPDATE
                race_candidates rc
            SET
                votes = COALESCE(first_choice_votes,
                    results.votes_for_candidate::integer)
            FROM
                results
            WHERE
                rc.ref_key = results.ref_key
                AND rc.race_id = results.race_id
            RETURNING
                *
        ),
        update_race AS (
            UPDATE
                race
            SET
                total_votes = COALESCE(total_first_choice_votes,
                    NULLIF(results.total_number_of_votes_for_office_in_area::integer,
                        0)),
                num_precincts_reporting = results.number_of_precincts_reporting::integer,
                total_precincts = results.total_number_of_precincts_voting_for_the_office::integer
            FROM
                results
            WHERE
                race.id = results.race_id
        )
        SELECT
            COUNT(*)::bigint AS result_rows,
            COUNT(race_id)::bigint AS matched_rows,
            STRING_AGG(
                CONCAT(office_name, ' | ', candidate_name),
                '; ' ORDER BY office_name, candidate_name
            ) FILTER (WHERE race_id IS NULL) AS unmatched_candidates
        FROM results;
    "#,
        ref_key_values, source_tables, ELECTION_ID
    );

    let mut query = sqlx::query_as::<_, UpdateSummary>(&query);
    for candidate in &ref_key_candidates {
        query = query
            .bind(&candidate.office_name)
            .bind(&candidate.candidate_name)
            .bind(&candidate.ref_key)
            .bind(candidate.priority);
    }
    let summary = query.fetch_one(&db_pool.connection).await?;

    let unmatched_rows = summary.result_rows - summary.matched_rows;
    if unmatched_rows > 0 {
        return Err(std::io::Error::other(format!(
            "Updated matched Minnesota results, but {} of {} candidate rows did not match: {}",
            unmatched_rows,
            summary.result_rows,
            summary.unmatched_candidates.as_deref().unwrap_or("unknown")
        ))
        .into());
    }

    println!(
        "Public schema successfully updated with results: {}/{} candidate rows matched",
        summary.matched_rows, summary.result_rows
    );
    Ok(())
}

fn _write_to_csv_file(name: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(format!("{}.csv", name))?;
    std::io::Write::write_all(&mut file, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ELECTION_SLUG: &str = "minnesota-primaries-2026";

    #[test]
    fn decodes_windows_1252_candidate_names() {
        let encoded = b"Mar\xeda Isa P\xe9rez-Vega;Am\xe1da M\xe1rquez Simula;Catarina G\xf3mez;Wal\xe9 Elegbede";

        assert_eq!(
            decode_results_text(encoded),
            "María Isa Pérez-Vega;Amáda Márquez Simula;Catarina Gómez;Walé Elegbede"
        );
    }

    #[test]
    fn result_keys_cover_current_sos_and_filing_name_differences() {
        let cases = [
            (
                "County Commissioner District 5",
                "María Isa Pérez-Vega",
                "mn-sos-minnesota-primaries-2026-county-commissioner-district-5-maria-isa-perez-vega",
            ),
            (
                "State Representative District 39B",
                "Amáda Márquez Simula",
                "mn-sos-minnesota-primaries-2026-state-representative-district-39b-amada-marquez-simula",
            ),
            (
                "Council Member (Inver Grove Heights) (Elect 2)",
                "Mary T'Kach",
                "mn-sos-minnesota-primaries-2026-council-member-inver-grove-heights-elect-2-mary-t-kach",
            ),
            (
                "State Representative District 29B",
                "Marion (O'Neill) Rarick",
                "mn-sos-minnesota-primaries-2026-state-representative-district-29b-marion-o-neill-rarick",
            ),
            (
                "Council Member (Burnsville) (Elect 2)",
                "Catarina \"Cati\" Gómez",
                "mn-sos-minnesota-primaries-2026-council-member-burnsville-elect-2-catarina-cati-gomez",
            ),
            (
                "County Auditor/Treasurer",
                "Amy Rosing",
                "mn-sos-minnesota-primaries-2026-county-auditor-treasurer-amy-rosing",
            ),
            (
                "U.S. Senator",
                "Ahmad R. (Raafat) Hassan",
                "mn-sos-minnesota-primaries-2026-u-s-senator-ahmad-r-hassan",
            ),
            (
                "County Auditor/Treasurer",
                "Kim Frederick",
                "mn-sos-minnesota-primaries-2026-county-auditor-treasurer-kim-frederick",
            ),
            (
                "County Auditor/Treasurer",
                "Zac Baer",
                "mn-sos-minnesota-primaries-2026-county-auditor-treasurer-zac-baer",
            ),
            (
                "Mayor (Rochester)",
                "Walé Elegbede",
                "mn-sos-minnesota-primaries-2026-mayor-rochester-wale-elegbede",
            ),
            (
                "County Auditor/Treasurer",
                "Nathan Martin",
                "mn-sos-minnesota-primaries-2026-county-auditor-treasurer-nathan-martin",
            ),
            (
                "County Commissioner District 2",
                "Danny O'Keefe",
                "mn-sos-minnesota-primaries-2026-county-commissioner-district-2-danny-o-keefe",
            ),
            (
                "County Auditor/Treasurer",
                "Darin O Halvorson",
                "mn-sos-minnesota-primaries-2026-county-auditor-treasurer-darin-o-halvorson",
            ),
            (
                "County Auditor/Treasurer",
                "Brent Benscoter",
                "mn-sos-minnesota-primaries-2026-county-auditor-treasurer-brent-benscoter",
            ),
        ];

        for (office_name, candidate_name, expected_ref_key) in cases {
            let candidates = result_ref_key_candidates(ELECTION_SLUG, office_name, candidate_name);
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate == expected_ref_key),
                "No generated key matched {expected_ref_key}; generated {candidates:?}"
            );
        }

        let bill_gates_keys =
            result_ref_key_candidates(ELECTION_SLUG, "Governor", "Bill E Gates J.R.");
        assert!(bill_gates_keys
            .iter()
            .any(|key| key.ends_with("bill-e-gates-j-r")));
        assert!(bill_gates_keys
            .iter()
            .any(|key| key.ends_with("bill-e-gates-jr")));
    }

    #[test]
    fn normalizes_governor_tickets_and_ignores_write_ins() {
        assert_eq!(
            normalized_result_identities(
                "Governor & Lt Governor",
                "Amy Klobuchar and Ben Schierer"
            ),
            vec![
                ("Governor".to_string(), "Amy Klobuchar".to_string()),
                (
                    "Lieutenant Governor".to_string(),
                    "Ben Schierer".to_string()
                )
            ]
        );
        assert!(normalized_result_identities("County Commissioner", "WRITE-IN").is_empty());
    }
}
