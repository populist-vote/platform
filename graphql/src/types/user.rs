use super::{AddressResult, AddressExtendedMNResult};
use crate::{context::ApiContext, guard::UserGuard, is_admin};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use db::{models::enums::State, Address, AddressExtendedMN, UserWithProfile};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct UserResult {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_picture_url: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateUserProfileInput {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[ComplexObject]
impl UserResult {
    #[graphql(guard = "UserGuard::new(&self.id)", visible = "is_admin")]
    async fn address(&self, ctx: &Context<'_>) -> Result<Option<AddressResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(Address,
            r#"
            SELECT a.id, a.line_1, a.line_2, a.city, a.county, a.state AS "state:State", a.postal_code, a.country, a.congressional_district, a.state_senate_district, a.state_house_district FROM address AS a
            JOIN user_profile up ON user_id = $1
            JOIN address ON up.address_id = a.id
        "#,
            uuid::Uuid::parse_str(&self.id)?,
        )
        .fetch_optional(&db_pool)
        .await.unwrap();

        match record {
            Some(address) => Ok(Some(address.into())),
            None => Ok(None),
        }
    }

    #[graphql(guard = "UserGuard::new(&self.id)", visible = "is_admin")]
    async fn address_extended_mn(&self, ctx: &Context<'_>) -> Result<Option<AddressExtendedMNResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query_as!(AddressExtendedMN,
            r#"
            SELECT gid, vtdid as voting_tabulation_district_id,
            countycode as county_code, countyname as county_name,
            pctcode as precinct_code, pctname as precinct_name,
            ctycomdist as county_commissioner_district
            FROM p6t_state_mn.bdry_votingdistricts as vt
            JOIN user_profile up ON up.user_id = $1
            JOIN address a ON up.address_id = a.id
            WHERE ST_Contains(ST_SetSRID(vt.geom, 26915), ST_Transform(a.geom, 26915))
        "#,
            uuid::Uuid::parse_str(&self.id)?,
        )
        .fetch_optional(&db_pool)
        .await.unwrap();

        match record {
            Some(address_extended_mn) => Ok(Some(address_extended_mn.into())),
            None => Ok(None),
        }
    }
}

impl From<UserWithProfile> for UserResult {
    fn from(user: UserWithProfile) -> Self {
        Self {
            id: user.id.into(),
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            profile_picture_url: user.profile_picture_url,
        }
    }
}
