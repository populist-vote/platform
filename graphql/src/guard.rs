use async_graphql::{Context, Guard, Result, ID};
use auth::AccessTokenClaims;
use jsonwebtoken::TokenData;
use uuid::Uuid;

use crate::context::ApiContext;

// Could genericize and expand this struct to take a role (for gating certain API calls to certain roles, e.g.)
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
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<AccessTokenClaims>>>() {
            let user_roles = token_data.claims.roles;
            // Check that user has staff or superuser role for populist organization
            if let Some(user_roles) = user_roles {
                let populist = sqlx::query!(
                    r#"
                        SELECT id FROM organization WHERE slug = 'populist'
                    "#
                )
                .fetch_one(&ctx.data::<ApiContext>()?.pool)
                .await?;
                if user_roles.iter().any(|r| {
                    r.organization_id == populist.id
                        && (r.role == db::Role::STAFF || r.role == db::Role::SUPERUSER)
                }) {
                    Ok(())
                } else {
                    Err("You don't have permission to to run this query/mutation".into())
                }
            } else {
                Err("You don't have permission to to run this query/mutation".into())
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
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<AccessTokenClaims>>>() {
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

pub struct OrganizationGuard<'a> {
    organization_id: &'a ID,
}

impl<'a> OrganizationGuard<'a> {
    pub fn new(organization_id: &'a ID) -> Self {
        Self { organization_id }
    }
}

#[async_trait::async_trait]
impl<'a> Guard for OrganizationGuard<'a> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<AccessTokenClaims>>>() {
            let user_roles = token_data.claims.roles;
            if let Some(user_roles) = user_roles {
                if user_roles.iter().any(|r| {
                    r.organization_id == Uuid::parse_str(self.organization_id.as_str()).unwrap()
                }) {
                    Ok(())
                } else {
                    Err("You don't have permission to to run this query/mutation".into())
                }
            } else {
                return Err("You don't have permission to to run this query/mutation".into());
            }
        } else {
            Err("You don't have permission to to run this query/mutation".into())
        }
    }
}
