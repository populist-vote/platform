use async_graphql::{Context, Guard, Result, ID};
use auth::Claims;
use jsonwebtoken::TokenData;
use uuid::Uuid;

// Could genericize and expand this struct to take a role (for gating certain API calls to certains roles, e.g.)
//
// pub struct UserGuard;
// impl UserGuard {
//     pub fn new(role: Option<Role>, tenant_id: Option<uuid::Uuid>) -> UserGuard {
//         UserGuard { role, tenant_id }
//     }
// }
pub struct StaffOnly;

#[async_trait::async_trait]
impl Guard for StaffOnly {
    async fn check(&self, ctx: &Context<'_>) -> Result<(), async_graphql::Error> {
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<Claims>>>() {
            match token_data.claims.role {
                db::Role::STAFF => Ok(()),
                db::Role::SUPERUSER => Ok(()),
                _ => Err("You don't have permission to to run this query/mutation".into()),
            }
        } else {
            Err("You don't have permission to to run this query/mutation".into())
        }
    }
}

pub struct UserGuard<'a> {
    id: &'a ID,
}

impl<'a> UserGuard<'a> {
    pub fn new(id: &'a ID) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl<'a> Guard for UserGuard<'a> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<Claims>>>() {
            if token_data.claims.sub == Uuid::parse_str(self.id.as_str())? {
                Ok(())
            } else {
                Err("You don't have permission to to run this query/mutation".into())
            }
        } else {
            Err("You don't have permission to to run this query/mutation".into())
        }
    }
}
