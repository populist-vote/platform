use async_graphql::{Context, Object, Result, SimpleObject, ID};
use auth::{create_access_token_for_user, Claims};
use db::{AddressInput, Role, User};
use http::header::SET_COOKIE;
use jsonwebtoken::TokenData;

use crate::{
    context::ApiContext,
    is_admin,
    types::{AddressResult, Error},
};

#[derive(Default)]
pub struct UserMutation;

#[derive(SimpleObject)]
struct UpdateUsernameResult {
    pub username: String,
}

#[derive(SimpleObject)]
struct UpdateEmailResult {
    pub email: String,
}

#[derive(SimpleObject)]
struct UpdateNameResult {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[Object]
impl UserMutation {
    #[graphql(visible = "is_admin")]
    async fn update_username(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> Result<UpdateUsernameResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;
        let updated_record = sqlx::query_as!(
            User,
            r#"
            UPDATE populist_user SET username = $1
            WHERE id = $2
            RETURNING id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at
        "#,
            username,
            user_id,
        )
        .fetch_one(&db_pool)
        .await;

        match updated_record {
            Ok(user) => {
                let access_token = create_access_token_for_user(user.clone())?;
                ctx.insert_http_header(
                    SET_COOKIE,
                    format!(
                        "access_token={}; HttpOnly; SameSite=None; Secure",
                        access_token
                    ),
                );
                Ok(UpdateUsernameResult {
                    username: user.username,
                })
            }
            Err(err) => match err {
                sqlx::Error::RowNotFound => Err(Error::UserNotFound.into()),
                sqlx::Error::Database(err)
                    if err.constraint() == Some("populist_user_username_key") =>
                {
                    Err(Error::UsernameTaken.into())
                }
                _ => Err(err.into()),
            },
        }
    }

    #[graphql(visible = "is_admin")]
    async fn update_email(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(email))] email: String,
    ) -> Result<UpdateEmailResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;
        let updated_record = sqlx::query_as!(
            User,
            r#"
            UPDATE populist_user SET email = $1
            WHERE id = $2
            RETURNING id, email, username, password, role AS "role:Role", created_at, confirmed_at, updated_at
        "#,
            email,
            user_id,
        )
        .fetch_one(&db_pool)
        .await;

        match updated_record {
            Ok(user) => {
                let access_token = create_access_token_for_user(user.clone())?;
                ctx.insert_http_header(
                    SET_COOKIE,
                    format!(
                        "access_token={}; HttpOnly; SameSite=None; Secure",
                        access_token
                    ),
                );
                Ok(UpdateEmailResult { email: user.email })
            }
            Err(err) => match err {
                sqlx::Error::RowNotFound => Err(Error::UserNotFound.into()),
                sqlx::Error::Database(err)
                    if err.constraint() == Some("populist_user_email_key") =>
                {
                    Err(Error::UserExistsError.into())
                }
                _ => Err(err.into()),
            },
        }
    }

    #[graphql(visible = "is_admin")]
    async fn update_first_and_last_name(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 1))] first_name: String,
        #[graphql(validator(min_length = 1))] last_name: String,
    ) -> Result<UpdateNameResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;
        let updated_record = sqlx::query!(
            r#"
            UPDATE user_profile SET first_name = $1, last_name = $2
            WHERE user_id = $3
            RETURNING first_name, last_name
        "#,
            first_name,
            last_name,
            user_id,
        )
        .fetch_one(&db_pool)
        .await;

        match updated_record {
            Ok(user_profile) => Ok(UpdateNameResult {
                first_name: user_profile.first_name,
                last_name: user_profile.last_name,
            }),
            Err(err) => match err {
                sqlx::Error::RowNotFound => Err(Error::UserNotFound.into()),
                _ => Err(err.into()),
            },
        }
    }

    #[graphql(visible = "is_admin")]
    async fn update_address(
        &self,
        ctx: &Context<'_>,
        address: AddressInput,
    ) -> Result<AddressResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;

        let result = User::update_address(&db_pool, user_id, address).await?;

        Ok(result.into())
    }

    #[graphql(visible = "is_admin")]
    async fn delete_account(&self, ctx: &Context<'_>) -> Result<ID> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<Claims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;

        let result = sqlx::query!(
            r#"
            DELETE FROM populist_user WHERE id = $1
            RETURNING id
        "#,
            user_id
        )
        .fetch_one(&db_pool)
        .await?;

        ctx.insert_http_header(
            SET_COOKIE,
            format!(
                "access_token=null; expires={}; Max-Age=0; HttpOnly; SameSite=None; Secure",
                (chrono::Utc::now() - chrono::Duration::hours(1)).format("%a, %d %b %Y %T GMT")
            ),
        );

        Ok(result.id.into())
    }
}
