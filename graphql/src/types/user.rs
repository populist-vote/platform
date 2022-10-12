use super::{AddressExtendedMNResult, AddressResult};
use crate::{context::ApiContext, guard::UserGuard, is_admin};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use db::{Address, UserWithProfile};

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
        let address =
            Address::find_by_user_id(&db_pool, &uuid::Uuid::try_parse(&self.id).unwrap()).await?;
        Ok(address.map(|address| address.into()))
    }

    #[graphql(guard = "UserGuard::new(&self.id)", visible = "is_admin")]
    async fn address_extended_mn(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<AddressExtendedMNResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let address =
            Address::extended_mn_by_user_id(&db_pool, &uuid::Uuid::try_parse(&self.id).unwrap())
                .await?;
        Ok(address.map(|address| address.into()))
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
