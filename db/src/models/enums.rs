use async_graphql::Enum;
use strum_macros::{Display, EnumString};

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, sqlx::Type)]
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

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, sqlx::Type)]
#[strum(ascii_case_insensitive)]
#[sqlx(type_name = "political_party", rename_all = "lowercase")]
pub enum PoliticalParty {
    DEMOCRATIC,
    REPUBLICAN,
    LIBERTARIAN,
    GREEN,
    CONSTITUTION,
    UNKNOWN,
}

impl Default for PoliticalParty {
    fn default() -> Self {
        PoliticalParty::UNKNOWN
    }
}

#[derive(Enum, Debug, Display, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "author_type", rename_all = "lowercase")]
pub enum AuthorType {
    POLITICIAN,
    ORGANIZATION,
}

#[derive(Enum, Display, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "argument_position", rename_all = "lowercase")]
pub enum ArgumentPosition {
    SUPPORT,
    NEUTRAL,
    OPPOSE,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "vote_status", rename_all = "lowercase")]
pub enum LegislationStatus {
    INTRODUCED,
    PASSED,
    SIGNED,
    VETOED,
    UNKNOWN,
}

impl Default for LegislationStatus {
    fn default() -> Self {
        LegislationStatus::INTRODUCED
    }
}
