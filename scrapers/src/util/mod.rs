use encoding_rs::WINDOWS_1252;
use project_root::get_project_root;
use spinoff::{spinners, Color, Spinner, Streams};
use std::time::{Duration, Instant};
use std::{error::Error, fs};
pub mod extensions;

/// Decode bytes to a valid UTF-8 string. Tries UTF-8 first; if invalid, decodes as Windows-1252
/// (common for US county/state CSV exports; superset of ISO-8859-1 in 0x80–0x9F, so curly quotes
/// and similar characters decode correctly) and returns the UTF-8 representation.
pub fn decode_csv_bytes_to_utf8(bytes: &[u8]) -> String {
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return s;
    }
    let (cow, _, _) = WINDOWS_1252.decode(bytes);
    cow.into_owned()
}

pub fn read_local_html(path: &str) -> Result<String, Box<dyn Error>> {
    let path = get_project_root()?.join("scrapers/html").join(path);
    let html = fs::read_to_string(path)?;
    Ok(html)
}

pub async fn run_with_timer<F, Fut>(message: String, task: F) -> Result<Duration, Box<dyn Error>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn Error>>>,
{
    let mut spinner =
        Spinner::new_with_stream(spinners::Arc, message, Color::Green, Streams::Stdout);
    let start = Instant::now();
    let result = task().await;
    let duration = start.elapsed();
    if result.is_ok() {
        spinner.stop_and_persist("✅", &format!("Success — 🕑 {:?}", duration));
    } else {
        spinner.stop_and_persist("❌", &format!("Failed — 🕑 {:?}", duration));
    }
    result.map(|_| duration)
}
