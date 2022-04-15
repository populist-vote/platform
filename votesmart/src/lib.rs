mod api;
mod errors;
mod types;
pub use api::*;
pub use errors::*;
pub use types::*;

const VOTESMART_BASE_URL: &str = "http://api.votesmart.org/";

/// Stuct used to make calls to the Votesmart API
pub struct VotesmartProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
}

impl VotesmartProxy {
    /// Instantiate new VotesmartProxy API client from .env api key
    pub fn new() -> Result<Self, Error> {
        dotenv::dotenv().ok();
        let api_key = std::env::var("VOTESMART_API_KEY")?;
        let client = reqwest::Client::new();

        Ok(VotesmartProxy {
            client,
            base_url: reqwest::Url::parse(VOTESMART_BASE_URL).unwrap(),
            api_key,
        })
    }

    /// Instantiate new VotesmartProxy API client by passing api key to this function
    pub fn new_from_key(api_key: String) -> Result<Self, Error> {
        let client = reqwest::Client::new();

        Ok(VotesmartProxy {
            client,
            base_url: reqwest::Url::parse(VOTESMART_BASE_URL).unwrap(),
            api_key,
        })
    }
}

/// Endpoint function namespaces.
impl VotesmartProxy {
    /// Office endpoint methods.
    pub const fn office(&self) -> Office<'_> {
        Office(self)
    }
    /// Officials endpoint methods.
    pub const fn officials(&self) -> Officials<'_> {
        Officials(self)
    }
    /// Rating endpoint methods.
    pub const fn rating(&self) -> Rating<'_> {
        Rating(self)
    }
    /// State endpoint methods.
    pub const fn state(&self) -> State<'_> {
        State(self)
    }
    /// Address endpoint methods.
    pub const fn address(&self) -> Address<'_> {
        Address(self)
    }
    /// Candidates endpoint methods.
    pub const fn candidates(&self) -> Candidates<'_> {
        Candidates(self)
    }
    /// Committee endpoint methods.
    pub const fn committee(&self) -> Committee<'_> {
        Committee(self)
    }
    /// District endpoint methods.
    pub const fn district(&self) -> District<'_> {
        District(self)
    }
    /// Election endpoint methods.
    pub const fn election(&self) -> Election<'_> {
        Election(self)
    }
    /// Leadership endpoint methods.
    pub const fn leadership(&self) -> Leadership<'_> {
        Leadership(self)
    }
    /// Vote endpoint methods.
    pub const fn votes(&self) -> Votes<'_> {
        Votes(self)
    }
    /// CandidateBio endpoint methods.
    pub const fn candidate_bio(&self) -> CandidateBio<'_> {
        CandidateBio(self)
    }
}
