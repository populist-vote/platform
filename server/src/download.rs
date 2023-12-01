use std::fs::File;

use axum::{
    body::StreamBody,
    extract::{Path, State},
    response::IntoResponse,
};
use csv::Writer;
use db::DatabasePool;
use http::{header, StatusCode};
use strum_macros::EnumString;
use tokio_util::io::ReaderStream;

#[derive(EnumString, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Dataset {
    CandidateFilings,
}

#[derive(serde::Deserialize, Debug)]
pub struct Params {
    year: i32,
    dataset: Dataset,
}

// Deserialize this from kebob case

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct CandidateFiling {
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    preferred_name: Option<String>,
}

#[axum::debug_handler]
pub async fn dataset_handler(
    Path(params): Path<Params>,
    State(pool): State<DatabasePool>,
) -> Result<impl IntoResponse, StatusCode> {
    let year_filter = params.year;
    let dataset = params.dataset;

    // Determine which dataset is requested
    match dataset {
        Dataset::CandidateFilings => {
            // Build query for dataset to get freshest data
            let query_result = sqlx::query_as!(
                CandidateFiling,
                r#"
                    SELECT first_name, middle_name, last_name, preferred_name
                    FROM politician p
                    JOIN race_candidates rc ON rc.candidate_id = p.id
                    JOIN race r ON r.id = rc.race_id
                    JOIN office o ON o.id = r.office_id
                    JOIN election e ON e.id = r.election_id
                    WHERE EXTRACT(YEAR FROM e.election_date)::INTEGER = $1
                "#,
                year_filter
            )
            .fetch_all(&pool.connection)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

            match query_result {
                Ok(data) => {
                    // Handle Vec<Record> to csv with helper fn
                    let file_path = "candidate_filings.csv";
                    let csv = convert_to_csv(data, file_path)
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
                    if csv.is_err() {
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }

                    // For example, return a success message
                    Ok(download_csv(file_path).await)
                }
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// Function to convert Vec<Record> to CSV file
fn convert_to_csv<T>(records: Vec<T>, file_path: &str) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let file = File::create(file_path)?;
    let mut writer = Writer::from_writer(file);

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;

    Ok(())
}

async fn download_csv(path: &str) -> Result<impl IntoResponse, (StatusCode, String)> {
    match tokio::fs::File::open(&path).await {
        Ok(file) => {
            // convert the `AsyncRead` into a `Stream`
            let stream = ReaderStream::new(file);
            // convert the `Stream` into an `axum::body::HttpBody`
            let body = StreamBody::new(stream);

            let headers = [
                (
                    header::CONTENT_TYPE,
                    format!("{}; charset=utf-8", "text/csv"),
                ),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", path),
                ),
            ];

            Ok((headers, body))
        }
        Err(err) => Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    }
}
