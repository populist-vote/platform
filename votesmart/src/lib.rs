pub mod address;
pub mod candidate_bio;
pub mod candidates;
pub mod committee;
pub mod district;
pub mod election;
mod errors;
pub mod leadership;
pub mod office;
pub mod officials;
pub mod rating;
pub mod state;
mod types;
pub mod votes;
use address::Address;
use candidate_bio::CandidateBio;
use candidates::Candidates;
use committee::Committee;
use district::District;
use election::Election;
use errors::Error;
use leadership::Leadership;
use office::Office;
use officials::Officials;
use rating::Rating;
use state::State;
pub use types::{GetCandidateBioResponse, GetCandidateVotingRecordResponse};
use votes::Votes;

const VOTESMART_BASE_URL: &str = "http://api.votesmart.org/";

/// Stuct used to make calls to the Votesmart API
pub struct VotesmartProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
}

impl VotesmartProxy {
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
}

/// Endpoint function namespaces.
impl VotesmartProxy {
    /// Offic endpoint methods.
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
    /// Address endpont methods.
    pub const fn address(&self) -> Address<'_> {
        Address(self)
    }
    /// Candidates endpont methods.
    pub const fn candidates(&self) -> Candidates<'_> {
        Candidates(self)
    }
    /// Committee endpont methods.
    pub const fn committee(&self) -> Committee<'_> {
        Committee(self)
    }
    /// District endpont methods.
    pub const fn district(&self) -> District<'_> {
        District(self)
    }
    /// Election endpont methods.
    pub const fn election(&self) -> Election<'_> {
        Election(self)
    }
    /// Leadership endpont methods.
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
