use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Var(#[from] std::env::VarError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("Failed to fetch {0} from Legiscan API")]
    Api(String),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegiscanErrorResponse {
    pub status: String,
    pub alert: Alert,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    pub message: String,
}
