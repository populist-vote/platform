use async_graphql::{Context, Guard, Result, ID};
use auth::AccessTokenClaims;
use db::OrganizationRoleType;
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

impl Guard for StaffOnly {
    async fn check(&self, ctx: &Context<'_>) -> Result<(), async_graphql::Error> {
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<AccessTokenClaims>>>() {
            match token_data.claims.system_role {
                db::SystemRoleType::Staff => Ok(()),
                db::SystemRoleType::Superuser => Ok(()),
                _ => Err("You don't have permission to to run this query/mutation".into()),
            }
        } else {
            Err("You don't have permission to to run this query/mutation".into())
        }
    }
}

pub struct IntakeTokenGuard<'a> {
    intake_token: &'a str,
    slug: &'a str,
}

impl<'a> IntakeTokenGuard<'a> {
    pub fn new(intake_token: &'a str, slug: &'a str) -> Self {
        Self { intake_token, slug }
    }
}

impl<'a> Guard for IntakeTokenGuard<'a> {
    async fn check(&self, ctx: &Context<'_>) -> Result<(), async_graphql::Error> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let record = sqlx::query!(
            r#"
            SELECT id FROM politician WHERE intake_token = $1 AND slug = $2
        "#,
            self.intake_token,
            self.slug,
        )
        .fetch_optional(&db_pool)
        .await?;

        match record {
            Some(_) => Ok(()),
            None => {
                let staff_guard = StaffOnly;
                staff_guard.check(ctx).await
            }
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
    min_role: &'a OrganizationRoleType,
}

impl<'a> OrganizationGuard<'a> {
    pub fn new(organization_id: &'a ID, min_role: &'a OrganizationRoleType) -> Self {
        Self {
            organization_id,
            min_role,
        }
    }
}

impl<'a> Guard for OrganizationGuard<'a> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(token_data) = ctx.data_unchecked::<Option<TokenData<AccessTokenClaims>>>() {
            if token_data.claims.organizations.iter().any(|o| {
                o.organization_id == Uuid::parse_str(self.organization_id.as_str()).unwrap()
                    && o.role as i32 >= *self.min_role as i32
            }) {
                Ok(())
            } else {
                Err("You don't have permission to to run this query/mutation".into())
            }
        } else {
            Err("You don't have permission to to run this query/mutation".into())
        }
    }
}
