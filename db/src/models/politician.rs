use crate::models::{organization::Organization, user::User};
use crate::{DateTime, Id};
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Politician {
    pub id: Id,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub preferred_name: Option<String>,
    pub ballot_name: Option<String>,
    pub description: Option<String>,
    pub home_state: State,
    pub endorsements: Vec<Organization>,
    pub created_by: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone)]
pub enum State {
    AL,
    AK,
    AZ,
    AR,
    CA,
    CO,
    CT,
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

impl Politician {
    // async fn create(ctx: &Context, user_id: Id, )
}
