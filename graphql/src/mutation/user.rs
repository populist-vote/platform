use crate::{
    context::ApiContext,
    mutation::StaffOnly,
    types::{CreateUserResult, Error, LoginResult},
};
use async_graphql::*;
use auth::create_token_for_user;
use db::{CreateUserInput, User};
use pwhash::bcrypt;

#[derive(InputObject)]
pub struct LoginInput {
    email_or_username: String,
    password: String,
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    #[graphql(guard = "StaffOnly")]
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: CreateUserInput,
    ) -> Result<CreateUserResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let new_record = User::create(&db_pool, &input).await?;

        Ok(CreateUserResult::from(new_record))
    }

    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResult, Error> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let user_record =
            User::find_by_email_or_username(&db_pool, input.email_or_username).await?;

        let password_is_valid = bcrypt::verify(input.password, &user_record.password);

        if password_is_valid {
            let token = create_token_for_user(user_record)?;
            Ok(LoginResult {
                access_token: token,
            })
        } else {
            Err(Error::PasswordError)
        }
    }
}
