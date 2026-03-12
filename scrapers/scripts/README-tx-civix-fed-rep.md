# Texas Civix Republican Federal Primary Results

Scrape [Civix Election Night Results](https://goelect.txelections.civixapps.com/ivis-enr-ui/races) (Republican Federal primary) and write `data/tx/sos/sos-results-fed-rep.csv` with columns:

- **Race** – office/race name  
- **Choice** – candidate name  
- **Party** – `REP`  
- **votes_for_candidate** – total votes for that candidate  
- **total_votes** – race total (same for all rows in that race)

## Running the scraper

Start ChromeDriver, then run the scraper:

```bash
# Terminal 1
chromedriver --port=9515

# Terminal 2, from platform/scrapers
cargo run --bin tx_scrape_civix_fed_rep
```

Output: `data/tx/sos/sos-results-fed-rep.csv`

The Civix site is a JavaScript SPA; the scraper loads the page in a headless browser and extracts the race/candidate table. If the site layout changes, selectors may need updating.
