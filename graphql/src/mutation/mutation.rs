use super::{
    argument::ArgumentMutation, ballot_measure::BallotMeasureMutation, bill::BillMutation,
    election::ElectionMutation, issue_tag::IssueTagMutation, organization::OrganizationMutation,
    politician::PoliticianMutation, user::UserMutation,
};
use async_graphql::{Context, Guard, MergedObject, Result};
#[derive(MergedObject, Default)]
pub struct Mutation(
    ArgumentMutation,
    PoliticianMutation,
    OrganizationMutation,
    BillMutation,
    BallotMeasureMutation,
    ElectionMutation,
    IssueTagMutation,
    UserMutation,
);

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
        if let Some(token) = ctx.data::<Option<String>>().unwrap() {
            if let Ok(token_data) = auth::validate_token(token) {
                match token_data.claims.role {
                    db::Role::STAFF => Ok(()),
                    db::Role::SUPERUSER => Ok(()),
                    _ => Err("You don't have permission to to run this mutation".into()),
                }
            } else {
                Err("You don't have permission to to run this mutation".into())
            }
        } else {
            Err("You don't have permission to to run this mutation".into())
        }
    }
}
