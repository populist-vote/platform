use async_graphql::Enum;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    Enum,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    EnumString,
    EnumIter,
    sqlx::Type,
    Serialize,
    Deserialize,
)]
#[sqlx(type_name = "political_scope", rename_all = "lowercase")]
pub enum PoliticalScope {
    Local,
    State,
    Federal,
}

#[derive(
    Enum,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    EnumString,
    EnumIter,
    sqlx::Type,
    Serialize,
    Deserialize,
)]

pub enum State {
    AL,
    AK,
    AZ,
    AR,
    CA,
    CO,
    CT,
    DC,
    DE,
    FL,
    GA,
    HI,
    ID,
    IL,
    IN,
    IA,
    KS,
    KY,
    LA,
    ME,
    MD,
    MA,
    MI,
    MN,
    MS,
    MO,
    MT,
    NE,
    NV,
    NH,
    NJ,
    NM,
    NY,
    NC,
    ND,
    OH,
    OK,
    OR,
    PA,
    RI,
    SC,
    SD,
    TN,
    TX,
    UT,
    VT,
    VA,
    WA,
    WV,
    WI,
    WY,
}

impl Default for State {
    fn default() -> Self {
        State::CO
    }
}

trait FullState {
    fn full_state(&self) -> &str;
}

impl FullState for State {
    fn full_state(&self) -> &str {
        match self {
            State::AL => "Alabama",
            State::AK => "Alaska",
            State::AZ => "Arizona",
            State::AR => "Arkansas",
            State::CA => "California",
            State::CO => "Colorado",
            State::CT => "Connecticut",
            State::DC => "District of Columbia",
            State::DE => "Delaware",
            State::FL => "Florida",
            State::GA => "Georgia",
            State::HI => "Hawaii",
            State::ID => "Idaho",
            State::IL => "Illinois",
            State::IN => "Indiana",
            State::IA => "Iowa",
            State::KS => "Kansas",
            State::KY => "Kentucky",
            State::LA => "Louisiana",
            State::ME => "Maine",
            State::MD => "Maryland",
            State::MA => "Massachusetts",
            State::MI => "Michigan",
            State::MN => "Minnesota",
            State::MS => "Mississippi",
            State::MO => "Missouri",
            State::MT => "Montana",
            State::NE => "Nebraska",
            State::NV => "Nevada",
            State::NH => "New Hampshire",
            State::NJ => "New Jersey",
            State::NM => "New Mexico",
            State::NY => "New York",
            State::NC => "North Carolina",
            State::ND => "North Dakota",
            State::OH => "Ohio",
            State::OK => "Oklahoma",
            State::OR => "Oregon",
            State::PA => "Pennsylvania",
            State::RI => "Rhode Island",
            State::SC => "South Carolina",
            State::SD => "South Dakota",
            State::TN => "Tennessee",
            State::TX => "Texas",
            State::UT => "Utah",
            State::VT => "Vermont",
            State::VA => "Virginia",
            State::WA => "Washington",
            State::WI => "Wisconsin",
            State::WY => "Wyoming",
            State::WV => "West Virginia",
        }
    }
}

#[derive(
    Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, sqlx::Type, Serialize, Deserialize,
)]
#[strum(ascii_case_insensitive)]
#[sqlx(type_name = "political_party", rename_all = "lowercase")]
pub enum PoliticalParty {
    Democratic,
    Republican,
    Libertarian,
    Green,
    Constitution,
    Unknown,
}

impl Default for PoliticalParty {
    fn default() -> Self {
        PoliticalParty::Unknown
    }
}

#[derive(Enum, Debug, Display, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "author_type", rename_all = "lowercase")]
pub enum AuthorType {
    Politician,
    Organization,
}

#[derive(Enum, Display, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "argument_position", rename_all = "lowercase")]
pub enum ArgumentPosition {
    Support,
    Neutral,
    Oppose,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "legislation_status", rename_all = "snake_case")]
pub enum LegislationStatus {
    Introduced,
    PassedHouse,
    PassedSenate,
    FailedHouse,
    FailedSenate,
    ResolvingDifferences,
    SentToExecutive,
    BecameLaw,
    Vetoed,
    Unknown,
}

impl Default for LegislationStatus {
    fn default() -> Self {
        LegislationStatus::Introduced
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action_type", rename_all = "camelCase")]
pub enum LegislationAction {
    Introduced {
        date: chrono::DateTime<Utc>,
        sponsor_id: uuid::Uuid,
    },
    ReferredToCommittee {
        date: chrono::DateTime<Utc>,
        committee: String,
    },
    Amended {
        date: chrono::DateTime<Utc>,
        amendment_text: String,
    },
    CommitteeAction {
        date: chrono::DateTime<Utc>,
        committee_action_type: CommiteeActionType,
        committee_id: uuid::Uuid,
    },
    VoteAction {
        date: chrono::DateTime<Utc>,
        politician_id: uuid::Uuid,
        vote_action_type: VoteActionType,
    },
    BecameLawSigned {
        date: chrono::DateTime<Utc>,
        politician_id: uuid::Uuid,
    },
    BecameLawUnsigned {
        date: chrono::DateTime<Utc>,
        politician_id: uuid::Uuid,
    },
    Vetoed {
        date: chrono::DateTime<Utc>,
        politician_id: uuid::Uuid,
        /// A pocket veto occurs when a bill fails to become law because the
        /// president does not sign it within the ten-day period and cannot
        /// return the bill to Congress because Congress is no longer in session
        is_pocket_veto: bool,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommiteeActionType {
    Reported,
    Tabled,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VoteActionType {
    // A normal floor vote by House / Senate
    ChamberVote,
    /// When one chamber votes to agree to any changes from the other chamber
    ConcurrenceVote,
    /// When one chamber votes to disagree to changes from the other chamber -
    /// leads then to conference committee to reach a compromise, resulting in a Conference Report
    NonConcurrenceVote,
    /// A vote on the final version of legislation that has been agreed upon by both chambers
    ConferenceReportVote,
}
