use crate::{
    context::ApiContext,
    delete_from_s3, is_admin,
    types::{AddressResult, Error},
    upload_to_s3, File,
};
use async_graphql::{Context, Object, Result, SimpleObject, Upload, ID};
use auth::{create_access_token_for_user, format_auth_cookie, AccessTokenClaims, TokenType};
use db::{AddressInput, SystemRoleType, User};
use jsonwebtoken::TokenData;
use std::io::Read;

#[derive(Default)]
pub struct UserMutation;

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct UpdateUsernameResult {
    pub username: String,
}

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct UpdateEmailResult {
    pub email: String,
}

#[derive(SimpleObject)]
#[graphql(visible = "is_admin")]
struct UpdateNameResult {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

pub async fn refresh_access_token(ctx: &Context<'_>, user_id: uuid::Uuid) -> Result<bool> {
    let db_pool = ctx.data::<ApiContext>()?.pool.clone();
    let user = User::find_by_id(&db_pool, user_id).await?;
    let organization_roles = User::organization_roles(&db_pool, user_id).await?;
    let access_token = create_access_token_for_user(user, organization_roles)?;
    ctx.insert_http_header(
        "Set-Cookie",
        format_auth_cookie(auth::TokenType::Access, &access_token),
    );
    Ok(true)
}

#[Object]
impl UserMutation {
    #[graphql(visible = "is_admin")]
    async fn upload_profile_picture(&self, ctx: &Context<'_>, file: Upload) -> Result<String> {
        let user_id = ctx
            .data::<Option<TokenData<AccessTokenClaims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let upload = file.value(ctx).unwrap();
        let mut content = Vec::new();
        let filename = user_id.to_string();
        let mimetype = upload.content_type.clone();

        upload.into_read().read_to_end(&mut content).unwrap();
        let file_info = File {
            id: ID::from(uuid::Uuid::new_v4()),
            filename,
            content,
            mimetype,
        };
        let url = upload_to_s3(file_info, "user-assets/profile-pictures".to_string()).await?;
        // Append last modified date because s3 path will remain the same and we want browser to cache, but refresh the image
        let url = format!("{}{}{}", url, "?lastmod=", chrono::Utc::now().timestamp());

        let _query = sqlx::query!(
            r#"
            UPDATE user_profile SET profile_picture_url = $1
            WHERE user_id = $2
        "#,
            url,
            user_id
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(url)
    }

    async fn delete_profile_picture(&self, ctx: &Context<'_>) -> Result<bool> {
        let user_id = ctx
            .data::<Option<TokenData<AccessTokenClaims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let query = sqlx::query!(
            r#"
            WITH up AS (
                SELECT
                    profile_picture_url
                FROM
                    user_profile
                WHERE
                    user_id = $1
            )
            UPDATE user_profile SET profile_picture_url = NULL
            WHERE user_id = $1
            RETURNING (SELECT profile_picture_url FROM up)
        "#,
            user_id
        )
        .fetch_one(&db_pool)
        .await?;

        if let Some(url) = query.profile_picture_url {
            let url = url::Url::parse(&url).unwrap();
            let s3_path = url.path().to_string();
            if let Err(err) = delete_from_s3(s3_path).await {
                tracing::error!("Error deleting profile picture: {}", err);
            };
        };

        Ok(true)
    }

    #[graphql(visible = "is_admin")]
    async fn update_username(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> Result<UpdateUsernameResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<AccessTokenClaims>>>()
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
            RETURNING id, email, username, password, system_role AS "system_role:SystemRoleType", invited_at, refresh_token, created_at, confirmed_at, updated_at
        "#,
            username,
            user_id,
        )
        .fetch_one(&db_pool)
        .await;

        match updated_record {
            Ok(user) => {
                let organization_roles = User::organization_roles(&db_pool, user.id).await?;
                let access_token = create_access_token_for_user(user.clone(), organization_roles)?;
                ctx.insert_http_header(
                    "Set-Cookie",
                    format_auth_cookie(auth::TokenType::Access, &access_token),
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
            .data::<Option<TokenData<AccessTokenClaims>>>()
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
            RETURNING id, email, username, password, system_role AS "system_role:SystemRoleType", invited_at, refresh_token, created_at, confirmed_at, updated_at
        "#,
            email,
            user_id,
        )
        .fetch_one(&db_pool)
        .await;

        match updated_record {
            Ok(user) => {
                let organization_roles = User::organization_roles(&db_pool, user.id).await?;
                let access_token = create_access_token_for_user(user.clone(), organization_roles)?;
                ctx.insert_http_header(
                    "Set-Cookie",
                    format_auth_cookie(auth::TokenType::Access, &access_token),
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
            .data::<Option<TokenData<AccessTokenClaims>>>()
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
            .data::<Option<TokenData<AccessTokenClaims>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .claims
            .sub;

        let result = User::update_address(&db_pool, user_id, address).await?;

        refresh_access_token(ctx, user_id).await?;

        Ok(result.into())
    }

    #[graphql(visible = "is_admin")]
    async fn delete_account(&self, ctx: &Context<'_>) -> Result<ID> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let user_id = ctx
            .data::<Option<TokenData<AccessTokenClaims>>>()
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

        ctx.insert_http_header("Set-Cookie", format_auth_cookie(TokenType::Access, "null"));
        ctx.insert_http_header("Set-Cookie", format_auth_cookie(TokenType::Refresh, "null"));

        Ok(result.id.into())
    }

    #[graphql(visible = "is_admin")]
    async fn delete_account_by_email(&self, ctx: &Context<'_>, email: String) -> Result<ID> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();

        let result = sqlx::query!(
            r#"
            DELETE FROM populist_user WHERE email = $1
            RETURNING id
        "#,
            email
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(result.id.into())
    }
}
