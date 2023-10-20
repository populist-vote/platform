use async_graphql::Enum;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    Enum,
    Debug,
    Display,
    Default,
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
#[serde(rename_all = "lowercase")]
pub enum PoliticalScope {
    Local,
    State,
    #[default]
    Federal,
}

#[derive(
    Enum,
    Debug,
    Display,
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
    AS,
    AZ,
    AR,
    CA,
    CO,
    CT,
    DC,
    FM,
    DE,
    FL,
    GA,
    GU,
    HI,
    ID,
    IL,
    IN,
    IA,
    KS,
    KY,
    LA,
    ME,
    MH,
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
    MP,
    OH,
    OK,
    OR,
    PW,
    PA,
    PR,
    RI,
    SC,
    SD,
    TN,
    TX,
    UT,
    VT,
    VI,
    VA,
    WA,
    WV,
    WI,
    WY,
}

pub trait FullState {
    fn full_state(&self) -> &str;
}

impl FullState for State {
    fn full_state(&self) -> &str {
        match self {
            State::AL => "Alabama",
            State::AK => "Alaska",
            State::AS => "American Samoa",
            State::AZ => "Arizona",
            State::AR => "Arkansas",
            State::CA => "California",
            State::CO => "Colorado",
            State::CT => "Connecticut",
            State::DC => "District of Columbia",
            State::FM => "Federated States Of Micronesia",
            State::DE => "Delaware",
            State::FL => "Florida",
            State::GA => "Georgia",
            State::GU => "Guam",
            State::HI => "Hawaii",
            State::ID => "Idaho",
            State::IL => "Illinois",
            State::IN => "Indiana",
            State::IA => "Iowa",
            State::KS => "Kansas",
            State::KY => "Kentucky",
            State::LA => "Louisiana",
            State::ME => "Maine",
            State::MH => "Marshall Islands",
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
            State::MP => "Northern Mariana Islands",
            State::OH => "Ohio",
            State::OK => "Oklahoma",
            State::OR => "Oregon",
            State::PW => "Palau",
            State::PA => "Pennsylvania",
            State::PR => "Puerto Rico",
            State::RI => "Rhode Island",
            State::SC => "South Carolina",
            State::SD => "South Dakota",
            State::TN => "Tennessee",
            State::TX => "Texas",
            State::UT => "Utah",
            State::VT => "Vermont",
            State::VI => "Virgin Islands",
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
#[sqlx(type_name = "political_party", rename_all = "snake_case")]
pub enum PoliticalParty {
    #[serde(rename = "Colorado Center")]
    ColoradoCenter,
    Democratic,
    #[serde(rename = "Democratic Farmer Labor")]
    DemocraticFarmerLabor,
    #[serde(rename = "Grassroots Legalize Cannabis")]
    GrassrootsLegalizeCannabis,
    #[serde(rename = "Legal Marijuana Now")]
    LegalMarijuanaNow,
    Republican,
    Libertarian,
    Freedom,
    Unity,
    Green,
    #[serde(rename = "American Constitution")]
    AmericanConstitution,
    Constitution,
    Independent,
    #[serde(rename = "Approval Voting")]
    ApprovalVoting,
    #[serde(rename = "Socialist Workers")]
    SocialistWorkers,
    Unaffiliated,
    Unknown,
}

#[derive(
    Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, sqlx::Type, Serialize, Deserialize,
)]
#[strum(ascii_case_insensitive)]
#[sqlx(type_name = "race_type", rename_all = "lowercase")]
// #[serde(rename_all = "lowercase")]
pub enum RaceType {
    Primary,
    General,
}

#[derive(
    Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, sqlx::Type, Serialize, Deserialize,
)]
#[strum(ascii_case_insensitive)]
#[sqlx(type_name = "vote_type", rename_all = "lowercase")]
// #[serde(rename_all = "lowercase")]
pub enum VoteType {
    Plurality,
    RankedChoice,
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

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type, Default)]
#[sqlx(type_name = "bill_status", rename_all = "snake_case")]
pub enum BillStatus {
    Introduced,
    InConsideration,
    BecameLaw,
    Failed,
    Vetoed,
    #[default]
    Unknown,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type, Default)]
#[sqlx(type_name = "ballot_measure_status", rename_all = "snake_case")]
pub enum BallotMeasureStatus {
    Introduced,
    InConsideration,
    Proposed,
    GatheringSignatures,
    OnTheBallot,
    BecameLaw,
    Failed,
    #[default]
    Unknown,
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

#[derive(Enum, Display, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Chambers {
    All,
    House,
    Senate,
}

#[derive(
    Display,
    Enum,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    EnumString,
    sqlx::Type,
    Serialize,
    Deserialize,
    Default,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "chamber", rename_all = "lowercase")]
pub enum BillType {
    /// Most states have these types
    #[default]
    Bill,
    Resolution,

    /// Some states have these types
    JointResolution,
    ConcurrentResolution,
    Memorial,
    JointMemorial,
    ConcurrentMemorial,
    ConstitutionalAmendment,
    Proclamation,
    ExecutiveOrder,
    JointSessionResolution,
    RepealBill,
    Remonstration,
    StudyRequest,
    ConcurrentStudyRequest,

    /// New Hampshire
    Address,

    /// DC City Council
    CeremonialResolution,

    None,
}
