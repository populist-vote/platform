use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResponse {
    pub status: String,
    #[serde(rename = "searchresult")]
    pub search_results: serde_json::Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResults {
    pub summary: Summary,
    pub results: Vec<SearchResult>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub page: String,
    pub range: String,
    pub relevancy: String,
    pub count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub relevance: i64,
    pub state: String,
    pub bill_number: String,
    pub bill_id: i64,
    pub change_hash: String,
    pub url: String,
    pub text_url: String,
    pub research_url: String,
    pub last_action_date: String,
    pub last_action: String,
    pub title: String,
}

impl LegiscanProxy {
    /// Retrieve roll call vote information for a given roll_call_id
    // static
    pub async fn search(
        &self,
        state: &str,
        query: &str,
        year: Option<i32>,
        page: Option<i32>,
    ) -> Result<SearchResults, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&state={state}&query={query}&year={year}&page={page}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "search",
            state = state,
            query = query,
            year = year.unwrap_or(2020),
            page = page.unwrap_or(1)
        );
        let response = self.client.get(url).send().await.unwrap();
        let json: SearchResponse = response.json().await?;
        let summary = &json.search_results["summary"];
        let summary: Summary = serde_json::from_value(summary.to_owned()).unwrap();
        let results: Vec<SearchResult> = json
            .search_results
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, _val)| key.parse::<i32>().is_ok())
            .map(|(_key, value)| serde_json::from_value(value.to_owned()).unwrap())
            .collect();
        Ok(SearchResults { summary, results })
    }

    /// Retrieve results from the full text search engine (2000 results)
    // 1 hour
    pub async fn search_raw() {
        todo!()
    }
}

#[tokio::test]
async fn test_search() {
    let proxy = LegiscanProxy::new().unwrap();
    let search_results = proxy
        .search("CO", "JibberishStringWontMatch", None, None)
        .await
        .unwrap();
    assert_eq!(search_results.results.len(), 0);
    let search_results = proxy.search("CO", "tobacco", None, None).await.unwrap();
    assert_eq!(search_results.results.len() > 0, true);
    let search_results = proxy
        .search("CO", "marijuana", Some(2020), None)
        .await
        .unwrap();
    assert_eq!(
        search_results.results[0].title,
        "Social Equity Licensees In Regulated Marijuana"
    );
}
