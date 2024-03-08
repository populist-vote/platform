use csv::ReaderBuilder;
use std::error::Error;
use thirtyfour::prelude::*;

static HEADER_NAMES: [&str; 19] = [
    "office_code",
    "candidate_name",
    "office_id",
    "office_title",
    "county_id",
    "mcd_fips_code",
    "school_district_number",
    "party_abbreviation",
    "residence_street_address",
    "residence_city",
    "residence_state",
    "residence_zip",
    "campaign_address",
    "campaign_city",
    "campaign_state",
    "campaign_zip",
    "campaign_phone",
    "campaign_website",
    "campaign_email",
];

pub async fn get_mn_sos_candidate_filings_local() -> Result<(), Box<dyn Error>> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.goto("https://candidates.sos.mn.gov").await?;
    let link = driver
        .find(By::LinkText(
            "Candidate Filings - Local Offices (Municipal, School District, and Hospital District)",
        ))
        .await?;
    link.click().await?;
    let text = driver
        .find(By::XPath("/html/body/pre"))
        .await?
        .text()
        .await?;
    // Convert text to CSV semi colon delimited
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_reader(text.as_bytes());
    let mut csv_string = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut csv_string);
        // Write the headers from the above struct
        wtr.write_record(HEADER_NAMES)?;
        for result in reader.records() {
            let record = result?;
            wtr.write_record(&record)?;
        }
        wtr.flush()?;
    }

    // Write CSV to Populist Data Lake, with timestamp of last scrape

    // Write CSV data to Postgres table in p6t_state_mn schema

    let csv_data_as_string = String::from_utf8(csv_string)?;

    let copy_query =
        r#"COPY p6t_state_mn.mn_candidate_filings_local_2023 FROM STDIN WITH CSV HEADER;"#;
    let pool = db::pool().await;
    sqlx::query!(r#"DROP TABLE IF EXISTS p6t_state_mn.mn_candidate_filings_local_2023 CASCADE;"#)
        .execute(&pool.connection)
        .await?;
    let create_table_query = format!(
        "CREATE TABLE p6t_state_mn.mn_candidate_filings_local_2023 (
            {}
        );",
        HEADER_NAMES
            .iter()
            .map(|&name| format!("{} text", name))
            .collect::<Vec<String>>()
            .join(", ")
    );

    sqlx::query(&create_table_query)
        .execute(&pool.connection)
        .await?;
    // let mut tx = pool.connection.copy_in_raw(copy_query).await?;
    // tx.send(csv_data_as_string.as_bytes()).await?;
    // tx.finish().await?;
    driver.quit().await?;
    Ok(())
}
