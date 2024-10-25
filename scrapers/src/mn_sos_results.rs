use csv::ReaderBuilder;
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

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

pub async fn fetch_results() -> Result<(), Box<dyn Error>> {
    let mut results_file_paths: HashMap<&str, &str> = HashMap::new();
    results_file_paths.insert(
        "U.S. Senator Statewide",
        "https://electionresultsfiles.sos.state.mn.us/20241105/ussenate.txt",
    );
    results_file_paths.insert(
        "U.S. Representative by District",
        "https://electionresultsfiles.sos.state.mn.us/20241105/ushouse.txt",
    );
    results_file_paths.insert(
        "State Senator by District",
        "https://electionresultsfiles.sos.state.mn.us/20241105/stsenate.txt",
    );
    results_file_paths.insert(
        "County Races",
        "https://electionresultsfiles.sos.state.mn.us/20241105/cntyRaces.txt",
    );
    results_file_paths.insert(
        "Municipal Races and Questions",
        "https://electionresultsfiles.sos.state.mn.us/20241105/local.txt",
    );
    results_file_paths.insert(
        "School Board Races",
        "https://electionresultsfiles.sos.state.mn.us/20241105/sdrace.txt",
    );
    results_file_paths.insert(
        "State Representative by District",
        "https://electionresultsfiles.sos.mn.gov/20241105/LegislativeByDistrict.txt",
    );
    results_file_paths.insert(
        "District Court Judges",
        "https://electionresultsfiles.sos.mn.gov/20241105/judicialdst.txt",
    );

    let client = Client::new();
    for (name, url) in results_file_paths {
        let response = client.get(url).send().await?.text().await?;
        let data = convert_text_to_csv(name, &response);
        let csv_data_as_string = String::from_utf8(data.clone())?;
        let table_name = format!(
            "p6t_state_mn.results_2024_{}",
            name.replace(['.', ','], "")
                .replace(' ', "_")
                .to_lowercase()
        );
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
        // _write_to_csv_file(name, &data)?;
    }
    update_public_schema_with_results().await;

    Ok(())
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

fn convert_text_to_csv(name: &str, text: &str) -> Vec<u8> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_reader(text.as_bytes());
    let mut csv_string = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut csv_string);
        // Write the headers from the above struct
        if name == "Precinct Reporting Statistics" {
            wtr.write_record(PRECINCT_STATS_HEADER_NAMES).unwrap();
        } else {
            wtr.write_record(HEADER_NAMES).unwrap();
        }
        for result in reader.records() {
            // test that record is valid
            if result.is_err() {
                println!("Error reading record: {:?}", result);
                continue;
            }
            let record = result.unwrap();
            // test to ensure record has 16 parts
            if record.len() != 16 {
                continue;
            }
            wtr.write_record(&record)
                .unwrap_or_else(|_| panic!("Error writing record: {:?}", record))
        }
        wtr.flush().unwrap();
    }

    csv_string
}

async fn update_public_schema_with_results() {
    let db_pool = db::pool().await;
    let query = r#"
        WITH source AS (
            SELECT * FROM p6t_state_mn.results_2024_county_races
            UNION ALL
            SELECT * FROM p6t_state_mn.results_2024_municipal_races_and_questions
            UNION ALL
            SELECT * FROM p6t_state_mn.results_2024_school_board_races
            UNION ALL
            SELECT * FROM p6t_state_mn.results_2024_state_senator_by_district
            UNION ALL
            SELECT * FROM p6t_state_mn.results_2024_us_representative_by_district
            UNION ALL
            SELECT * FROM p6t_state_mn.results_2024_us_senator_statewide
            UNION ALL
            SELECT * FROM p6t_state_mn.results_2024_state_representative_by_district
            UNION ALL
            SELECT * FROM p6t_state_mn.results_2024_district_court_judges
        ),
        results AS (
            SELECT DISTINCT ON (office_name, candidate_name)
                office_name,
                source.office_id,
                candidate_name,
                votes_for_candidate,
                total_number_of_votes_for_office_in_area,
                number_of_precincts_reporting,
                total_number_of_precincts_voting_for_the_office,
                rc.race_id AS race_id,
                r.title AS race_title,
                r.vote_type AS vote_type,
                rc.votes AS race_candidate_votes,
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
                source
            LEFT JOIN race_candidates rc ON rc.ref_key = SLUGIFY(CONCAT('mn-sos-', source.candidate_name, '-', source.office_id, '-', source.county_id))
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
                rc.race_id = results.race_id
                AND rc.candidate_id = results.politician_id
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
            *
        FROM
            results
        WHERE
            office_name NOT ILIKE '%question%';
    "#;

    let result = sqlx::query(query)
        .execute(&db_pool.connection)
        .await
        .map_err(|e| {
            println!("Error updating public schema with results: {}", e);
            e
        });

    if result.is_ok() {
        println!("Public schema successfully updated with results");
    }
}

fn _write_to_csv_file(name: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(format!("{}.csv", name))?;
    std::io::Write::write_all(&mut file, data)?;
    Ok(())
}
