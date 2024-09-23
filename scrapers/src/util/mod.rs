use std::{error::Error, fs};

use project_root::get_project_root;

pub mod extensions;

pub fn read_local_html(path: &str) -> Result<String, Box<dyn Error>> {
    let path = get_project_root()?.join("scrapers/html").join(path);
    let html = fs::read_to_string(path)?;
    Ok(html)
}
