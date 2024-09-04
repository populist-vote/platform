use super::{AddressExtendedMNResult, AddressResult, OrganizationResult};
use crate::{context::ApiContext, guard::UserGuard, is_admin};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use db::{Address, Organization, UserWithProfile};

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
    async fn available_organizations(&self, ctx: &Context<'_>) -> Result<Vec<OrganizationResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let organizations = sqlx::query_as!(
            Organization,
            r#"
            SELECT
                o.*
            FROM
                organization o
            JOIN
                organization_users ou ON o.id = ou.organization_id
            WHERE
                ou.user_id = $1
                "#,
            &uuid::Uuid::try_parse(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await?;
        Ok(organizations
            .into_iter()
            .map(|organization| organization.into())
            .collect())
    }

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
        let address_id = sqlx::query!(
            r#"
            SELECT
                up.address_id
            FROM
                user_profile up
            WHERE
                up.user_id = $1
            "#,
            &uuid::Uuid::try_parse(&self.id).unwrap()
        )
        .fetch_one(&db_pool)
        .await?
        .address_id;

        if let Some(address_id) = address_id {
            let address = Address::extended_mn_by_address_id(&db_pool, &address_id).await?;
            Ok(address.map(|address| address.into()))
        } else {
            Ok(None)
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
