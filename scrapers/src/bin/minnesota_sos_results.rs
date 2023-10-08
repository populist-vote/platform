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

async fn get_mn_sos_results() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
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
            name.replace(" ", "_").to_lowercase()
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

        write_to_csv_file(name, &data)?;
    }

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
                .map(|&name| format!("{} text", name.replace(" ", "_").to_lowercase()))
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
            .map(|&name| format!("{} text", name.replace(" ", "_").to_lowercase()))
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
            wtr.write_record(&PRECINCT_STATS_HEADER_NAMES).unwrap();
        } else {
            wtr.write_record(&HEADER_NAMES).unwrap();
        }
        for result in reader.records() {
            let record = result.unwrap();
            wtr.write_record(&record).unwrap();
        }
        wtr.flush().unwrap();
    }

    csv_string
}

fn write_to_csv_file(name: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(format!("{}.csv", name))?;
    std::io::Write::write_all(&mut file, data)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = get_mn_sos_results().await {
        println!("error running example: {}", err);
    }
}
