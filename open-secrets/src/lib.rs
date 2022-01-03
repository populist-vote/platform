use reqwest::{Error, Response};
use std::fmt;

const OPEN_SECRETS_BASE_URL: &str = "http://opensecrets.org/api/";

#[derive(Debug)]
pub enum OutputType {
    Json,
    Xml,
    Doc,
}

// Lets default to JSON for our output
impl Default for OutputType {
    fn default() -> Self {
        OutputType::Json
    }
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OutputType::Json => write!(f, "json"),
            OutputType::Xml => write!(f, "xml"),
            OutputType::Doc => write!(f, "doc"),
        }
    }
}

/// Struct used to make calls to the OpenSecrets API
pub struct OpenSecretsProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
    output: OutputType,
}

impl OpenSecretsProxy {
    /// Instantiate new OpenSecretsProxy API client from .env api key
    pub fn new() -> Result<Self, std::env::VarError> {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OPEN_SECRETS_API_KEY")?;
        let client = reqwest::Client::new();

        Ok(OpenSecretsProxy {
            client,
            base_url: reqwest::Url::parse(OPEN_SECRETS_BASE_URL).unwrap(),
            api_key,
            output: OutputType::default(),
        })
    }

    /// Instantiate new VotesmartProxy API client by passing api key to this function
    pub fn new_from_key(api_key: String) -> Result<Self, Error> {
        let client = reqwest::Client::new();

        Ok(OpenSecretsProxy {
            client,
            base_url: reqwest::Url::parse(OPEN_SECRETS_BASE_URL).unwrap(),
            api_key,
            output: OutputType::default(),
        })
    }

    pub fn with_output(&self, output: OutputType) -> Self {
        Self {
            client: self.client.to_owned(),
            base_url: self.base_url.to_owned(),
            api_key: self.api_key.to_owned(),
            output,
        }
    }
}

/// OpenSecrets endpoint methods
impl OpenSecretsProxy {
    /// Provides a list of 117th Congressional legislators and associated attributes for specified subset (state or specific CID)
    ///
    /// # Arguments
    /// * `id` - (required) two character state code or specific CID
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn get_legislators(&self, id: &str) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&id={id}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "getLegislators",
            id = id,
            output = self.output.to_string()
        );
        self.client.get(url).send().await
    }

    /// Returns data on the personal finances of a member of Congress, as well as judicial and executive branches
    ///
    /// # Arguments
    /// * `cid` - (required) CRP CandidateID
    /// * `year` - 2013, 2014, 2015 and 2016 data provided where available
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn mem_pfd_profile(&self, cid: &str, year: Option<i32>) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&year={year}&cid={cid}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "memPFDprofile",
            year = match year {
                Some(year) => year.to_string(),
                None => "".to_string(),
            },
            cid = cid,
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Provides summary fundraising information for specified politician
    ///
    /// # Arguments
    /// * `cid` - (required) CRP CandidateID
    /// * `cycle` - (optional) 2012, 2014, 2016, 2018, 2020; use `None` for latest cycle
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn cand_summary(&self, cid: &str, cycle: Option<i32>) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&cid={cid}&cycle={cycle}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "candSummary",
            cid = cid,
            cycle = match cycle {
                Some(cycle) => cycle.to_string(),
                None => "".to_string(),
            },
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Returns top contributors to specified candidate for a House or Senate seat or member of Congress. These are 6-year numbers for senators/Senate candidates; 2-year numbers for representatives/House candidates.
    ///
    /// # Arguments
    /// * `cid` - (required) CRP CandidateID
    /// * `cycle` - (optional) 2012, 2014, 2016, 2018, 2020; use `None` for latest cycle
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn cand_contrib(
        &self,
        crp_candidate_id: &str,
        cycle: Option<i32>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&cid={cid}&cycle={cycle}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "candContrib",
            cid = crp_candidate_id,
            cycle = match cycle {
                Some(cycle) => cycle.to_string(),
                None => "".to_string(),
            },
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Provides the top ten industries contributing to a specified candidate for a House or Senate seat or member of Congress. These are 6-year numbers for Senators/Senate candidates; 2-year total for Representatives/House candidates.
    ///
    /// # Arguments
    /// * `cid` - (required) CRP CandidateID
    /// * `cycle` - 2012, 2014, 2016, 2018, 2020; use `None` for latest cycle
    pub async fn cand_industry(
        &self,
        crp_candidate_id: &str,
        cycle: Option<i32>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&cid={cid}&cycle={cycle}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "candIndustry",
            cid = crp_candidate_id,
            cycle = match cycle {
                Some(cycle) => cycle.to_string(),
                None => "".to_string(),
            },
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Provides total contributed to specified candidate from specified industry. Senate data reflects 2-year totals.
    ///
    /// # Arguments
    /// * `cid` - (required) CRP CandidateID
    /// * `cycle` - (optional) 2012, 2014, 2016, 2018, 2020; use `None` for latest cycle
    /// * `ind` - (required) a 3-character industry code
    pub async fn cand_ind_by_ind(
        &self,
        crp_candidate_id: &str,
        cycle: Option<i32>,
        ind: &str,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&cid={cid}&cycle={cycle}&ind={ind}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "candIndByInd",
            cid = crp_candidate_id,
            cycle = match cycle {
                Some(cycle) => cycle.to_string(),
                None => "".to_string(),
            },
            ind = ind,
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Provides sector total of specified politician's receipts
    ///
    /// # Arguments
    /// * `cid` - (required) CRP CandidateID
    /// * `cycle` - (optional) 2012, 2014, 2016, 2018, 2020; use `None` for latest cycle
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn cand_sector(&self, cid: &str, cycle: Option<i32>) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&cid={cid}&cycle={cycle}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "candSector",
            cid = cid,
            cycle = match cycle {
                Some(cycle) => cycle.to_string(),
                None => "".to_string(),
            },
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Provides summary fundraising information for a specific committee, industry and congress number
    ///
    /// # Arguments
    /// * `cmte` - (required) Committee ID in CQ format
    /// * `congo` - 112 (uses 2012 data), 113 (uses 2014 data), 114 (uses 2016 data), 115 (uses 2018 data), 116 (uses 2020 data); leave blank for latest congress
    /// * `indus` - (required) 3 char Industry code
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn cong_cmte_indus(
        &self,
        cmte: &str,
        congo: Option<i32>,
        indus: &str,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&congo={congo}&indus={indus}&cmte={cmte}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "congCmteIndus",
            cmte = cmte,
            congo = match congo {
                Some(congo) => congo.to_string(),
                None => "".to_string()
            },
            indus = indus,
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Provides a list of organizations and ids that match the term searched.
    ///
    /// # Arguments
    /// * `org` - (required) name or partial name of organization requested
    pub async fn get_orgs(&self, org: &str) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&org={org}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "getOrgs",
            org = org,
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Provides 2020 summary fundraising information for the specified organization id
    ///
    /// # Arguments
    /// * `crp_org_id` - (required) CRP orgid (available via getOrgID method)
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn org_summary(&self, crp_org_id: &str) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&id={id}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "orgSummary",
            id = crp_org_id,
            output = self.output
        );
        self.client.get(url).send().await
    }

    /// Method to access the latest 50 independent expenditure transactions reported. Updated 4 times a day.
    ///
    /// # Arguments
    /// * `output` - (optional) Output format, either json, xml, or doc; default is xml
    pub async fn independent_expend(&self) -> Result<Response, Error> {
        let url = format!(
            "{base_url}?method={method}&output={output}&apikey={key}",
            base_url = self.base_url,
            key = self.api_key,
            method = "independentExpend",
            output = self.output
        );
        self.client.get(url).send().await
    }
}

#[tokio::test]
async fn test_get_legislators() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.get_legislators("CO").await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let _json: serde_json::Value = response.json().await.unwrap();
}

#[tokio::test]
async fn test_mem_pfd_profile() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy
        .mem_pfd_profile("N00007360", Some(2016))
        .await
        .unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["response"]["member_profile"]["@attributes"]["name"],
        "Pelosi, Nancy"
    );
}

#[tokio::test]
async fn test_cand_summary() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.cand_summary("N00007360", None).await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    assert_eq!(
        json["response"]["summary"]["@attributes"]["cand_name"],
        "Pelosi, Nancy"
    );
}

#[tokio::test]
async fn test_cand_contrib() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.cand_contrib("N00007360", None).await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    assert_eq!(
        json["response"]["contributors"]["@attributes"]["cand_name"],
        "Nancy Pelosi (D)"
    );
}

#[tokio::test]
async fn test_cand_industry() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.cand_industry("N00007360", None).await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["response"]["industries"]["@attributes"]["cand_name"],
        "Nancy Pelosi (D)"
    );
}

#[tokio::test]
async fn test_cand_ind_by_ind() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy
        .cand_ind_by_ind("N00007360", None, "K02")
        .await
        .unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["response"]["candIndus"]["@attributes"]["cand_name"],
        "Pelosi, Nancy"
    );
}

#[tokio::test]
async fn test_cand_sector() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.cand_sector("N00007360", None).await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["response"]["sectors"]["@attributes"]["cand_name"],
        "Nancy Pelosi (D)"
    );
}

#[tokio::test]
async fn test_cong_cmte_indus() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy
        .cong_cmte_indus("HARM", Some(116), "F10")
        .await
        .unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["response"]["committee"]["@attributes"]["committee_name"],
        "HARM"
    );
}

#[tokio::test]
async fn test_get_orgs() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.get_orgs("Planned Parenthood").await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["response"]["organization"][0]["@attributes"]["orgname"],
        "International Planned Parenthood"
    );
}

#[tokio::test]
async fn test_org_summary() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.org_summary("D000022603").await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["response"]["organization"]["@attributes"]["orgname"],
        "International Planned Parenthood"
    );
}

#[tokio::test]
async fn test_independent_expend() {
    let proxy = OpenSecretsProxy::new().unwrap();
    let response = proxy.independent_expend().await.unwrap();
    assert_eq!(response.status().is_success(), true);
    let _json: serde_json::Value = response.json().await.unwrap();
}
