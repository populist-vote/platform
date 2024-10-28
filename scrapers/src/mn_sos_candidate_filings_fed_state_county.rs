use csv::ReaderBuilder;
use std::error::Error;
use thirtyfour::prelude::*;

static HEADER_NAMES: [&str; 21] = [
    "office_id",
    "candidate_name",
    "office_id_2",
    "office_title",
    "county_id",
    "other_id",
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
    "running_mate_website",
    "running_mate_email",
    "running_mate_phone",
];
static GENERAL_LINK_TEXT: &str =
    "Candidates in the General Election - Federal, State, and County Offices";
pub async fn get_mn_sos_candidate_filings_fed_state_county() -> Result<(), Box<dyn Error>> {
    let pool = db::pool().await;
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.goto("https://candidates.sos.mn.gov").await?;

    let link = driver.find(By::LinkText(GENERAL_LINK_TEXT)).await?;
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
    let csv_data_as_string = String::from_utf8(csv_string)?;

    sqlx::query!(
        r#"DROP TABLE IF EXISTS p6t_state_mn.mn_candidate_filings_fed_state_county_2024 CASCADE;"#
    )
    .execute(&pool.connection)
    .await?;
    let create_table_query = format!(
        "CREATE TABLE p6t_state_mn.mn_candidate_filings_fed_state_county_2024 (
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
    let mut tx = pool.connection.acquire().await?;
    let copy_query = r#"COPY p6t_state_mn.mn_candidate_filings_fed_state_county_2024 FROM STDIN WITH CSV HEADER;"#;
    let mut tx_copy = tx.copy_in_raw(copy_query).await?;
    tx_copy.send(csv_data_as_string.as_bytes()).await?;
    tx_copy.finish().await?;

    driver.quit().await?;
    Ok(())
}

static PRIMARY_HEADER_NAMES: [&str; 21] = [
    "office_id",
    "candidate_name",
    "office_id_2",
    "office_title",
    "county_id",
    "mn_party_id",
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
    "running_mate_website",
    "running_mate_email",
    "running_mate_phone",
];

static PRIMARY_LINK_TEXT: &str = "Candidates in the Primary - Federal, State, and County Offices";

pub async fn get_mn_sos_candidate_filings_fed_state_county_primaries() -> Result<(), Box<dyn Error>>
{
    let pool = db::pool().await;
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.goto("https://candidates.sos.mn.gov").await?;

    let link = driver.find(By::LinkText(PRIMARY_LINK_TEXT)).await?;
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
        wtr.write_record(PRIMARY_HEADER_NAMES)?;
        for result in reader.records() {
            let record = result?;
            if let Err(err) = wtr.write_record(&record) {
                eprintln!("Error writing record to CSV: {}", err);
            }
        }
        wtr.flush()?;
    }
    let csv_data_as_string = String::from_utf8(csv_string)?;

    sqlx::query!(
        r#"DROP TABLE IF EXISTS p6t_state_mn.mn_candidate_filings_fed_state_county_primaries_2024 CASCADE;"#
    )
    .execute(&pool.connection)
    .await?;
    let create_table_query = format!(
        "CREATE TABLE p6t_state_mn.mn_candidate_filings_fed_state_county_primaries_2024 (
        {}
    );",
        PRIMARY_HEADER_NAMES
            .iter()
            .map(|&name| format!("{} text", name))
            .collect::<Vec<String>>()
            .join(", ")
    );

    sqlx::query(&create_table_query)
        .execute(&pool.connection)
        .await?;
    let mut tx = pool.connection.acquire().await?;
    let copy_query = r#"COPY p6t_state_mn.mn_candidate_filings_fed_state_county_primaries_2024 FROM STDIN WITH CSV HEADER;"#;
    let mut tx_copy = tx.copy_in_raw(copy_query).await?;
    tx_copy.send(csv_data_as_string.as_bytes()).await?;
    tx_copy.finish().await?;

    driver.quit().await?;
    Ok(())
}
