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
        "County Races",
        "https://electionresultsfiles.sos.state.mn.us/20231107/cntyRaces.txt",
    );
    results_file_paths.insert(
        "County Races and Questions",
        "https://electionresultsfiles.sos.state.mn.us/20231107/cntyRaceQuestions.txt",
    );
    results_file_paths.insert(
        "City Questions",
        "https://electionresultsfiles.sos.state.mn.us/20231107/CityQuestions.txt",
    );
    results_file_paths.insert(
        "Municipal Races and Questions",
        "https://electionresultsfiles.sos.state.mn.us/20231107/local.txt",
    );
    results_file_paths.insert(
        "Municipal and School District Races and Questions by Precinct",
        "https://electionresultsfiles.sos.state.mn.us/20231107/localPrct.txt",
    );
    results_file_paths.insert(
        "School Board Races",
        "https://electionresultsfiles.sos.state.mn.us/20231107/sdrace.txt",
    );
    results_file_paths.insert(
        "School Referendum and Bond Questions",
        "https://electionresultsfiles.sos.state.mn.us/20231107/SchoolQuestions.txt",
    );
    results_file_paths.insert(
        "School Board Races and Questions",
        "https://electionresultsfiles.sos.state.mn.us/20231107/SDRaceQuestions.txt",
    );
    results_file_paths.insert(
        "County Races by Precinct",
        "https://electionresultsfiles.sos.state.mn.us/20231107/allracesbyprecinct.txt",
    );
    results_file_paths.insert(
        "Precinct Reporting Statistics",
        "https://electionresultsfiles.sos.state.mn.us/20231107/pctstats.txt",
    );

    let client = Client::new();
    for (name, url) in results_file_paths {
        let response = client.get(url).send().await?.text().await?;
        let data = convert_text_to_csv(name, &response);
        let csv_data_as_string = String::from_utf8(data.clone())?;
        let table_name = format!(
            "p6t_state_mn.results_2023_{}",
            name.replace(' ', "_").to_lowercase()
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
        let mut tx = pool.connection.copy_in_raw(&copy_query).await?;
        tx.send(csv_data_as_string.as_bytes()).await?;
        tx.finish().await?;
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
            let record = result.unwrap();
            wtr.write_record(&record).unwrap();
        }
        wtr.flush().unwrap();
    }

    csv_string
}

async fn update_public_schema_with_results() {
    let db_pool = db::pool().await;
    let query = r#"
        WITH source AS (
            SELECT * FROM p6t_state_mn.results_2023_county_races_and_questions
            UNION ALL 
            SELECT * FROM p6t_state_mn.results_2023_municipal_races_and_questions	
            UNION ALL SELECT * FROM p6t_state_mn.results_2023_school_board_races
        ),
        results AS (
            SELECT
                candidate_name,
                votes_for_candidate,
                total_number_of_votes_for_office_in_area,
                p.id AS politician_id,
                rc.race_id AS race_id,
                r.title AS race_title,
                r.vote_type AS vote_type,
                rc.votes AS race_candidate_votes,
                r.total_votes AS race_total_votes,
                CASE WHEN office_name ILIKE '%first choice%' THEN
                    json_build_object('votes', votes_for_candidate::int, 'total_votes', total_number_of_votes_for_office_in_area::int)
                ELSE
                    NULL
                END AS first_choice_votes,
                CASE WHEN office_name ILIKE '%second choice%' THEN
                    json_build_object('votes', votes_for_candidate::int, 'total_votes', total_number_of_votes_for_office_in_area::int)
                ELSE
                    NULL
                END AS second_choice_votes,
                CASE WHEN office_name ILIKE '%third choice%' THEN
                    json_build_object('votes', votes_for_candidate::int, 'total_votes', total_number_of_votes_for_office_in_area::int)
                ELSE
                    NULL
                END AS third_choice_votes,
                CASE WHEN office_name ILIKE '%fourth choice%' THEN
                    json_build_object('votes', votes_for_candidate::int, 'total_votes', total_number_of_votes_for_office_in_area::int)
                ELSE
                    NULL
                END AS fourth_choice_votes,
                CASE WHEN office_name ILIKE '%fifth choice%' THEN
                    json_build_object('votes', votes_for_candidate::int, 'total_votes', total_number_of_votes_for_office_in_area::int)
                ELSE
                    NULL
                END AS fifth_choice_votes
            FROM
                source
                JOIN politician p ON p.slug = SLUGIFY (source.candidate_name)
                JOIN race_candidates rc ON rc.candidate_id = p.id
                JOIN race r ON r.id = rc.race_id
            WHERE (
                SELECT slug 
                FROM election
                WHERE id = r.election_id
            ) = 'general-election-2023'
        ),
        update_race_candidates AS (
            UPDATE
                race_candidates rc
            SET
                votes = results.votes_for_candidate::integer,
                ranked_choice_results = CASE
                  WHEN first_choice_votes IS NOT NULL
                        OR second_choice_votes IS NOT NULL
                        OR third_choice_votes IS NOT NULL
                        OR fourth_choice_votes IS NOT NULL
                        OR fifth_choice_votes IS NOT NULL
                    THEN json_build_object('first_choice', first_choice_votes, 'second_choice', second_choice_votes, 'third_choice', third_choice_votes, 'fourth_choice', fourth_choice_votes, 'fifth_choice', fifth_choice_votes) 
                    ELSE NULL
                END
            FROM
                results
            WHERE
                rc.race_id = results.race_id
                AND rc.candidate_id = results.politician_id
        ),
        update_race AS (
            UPDATE
                race
            SET
                total_votes = NULLIF(results.total_number_of_votes_for_office_in_area::integer, 0)
            FROM
                results
            WHERE
                race.id = results.race_id
        )
        SELECT * FROM results;
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
