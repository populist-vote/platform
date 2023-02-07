use super::{
    address::{Address, AddressInput},
    enums::State,
};
use crate::{DateTime, Error};
use async_graphql::{Enum, InputObject};
use geocodio::GeocodioProxy;
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};

#[derive(FromRow, Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub password: String,
    pub role: Role,
    pub organization_id: Option<uuid::Uuid>,
    pub created_at: DateTime,
    pub confirmed_at: Option<DateTime>,
    pub updated_at: DateTime,
}

#[derive(FromRow, Debug, Clone)]
pub struct UserProfile {
    pub id: uuid::Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(FromRow, Debug, Clone)]
pub struct UserWithProfile {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_picture_url: Option<String>,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct CreateUserInput {
    #[graphql(validator(email))]
    pub email: String,
    pub username: String,
    pub password: String,
    pub role: Option<Role>,
    pub organization_id: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, InputObject)]
pub struct CreateUserWithProfileInput {
    #[graphql(validator(email))]
    pub email: String,
    pub username: String,
    pub password: String,
    pub address: AddressInput,
    pub confirmation_token: String,
}

#[derive(
    Debug, Clone, strum_macros::Display, Type, Serialize, Deserialize, Copy, Eq, PartialEq, Enum,
)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    SUPERUSER,
    STAFF,
    PREMIUM,
    BASIC,
}

impl User {
    pub async fn create(db_pool: &PgPool, input: &CreateUserInput) -> Result<Self, Error> {
        let hash = bcrypt::hash(&input.password).unwrap();
        let role = input.role.unwrap_or(Role::BASIC);
        let record = sqlx::query_as!(
            User,
            r#"
                INSERT INTO populist_user (email, username, password, role, organization_id)
                VALUES (LOWER($1), LOWER($2), $3, $4, $5)
                RETURNING id, email, username, password, role AS "role:Role", organization_id, created_at, confirmed_at, updated_at
            "#,
            input.email,
            input.username,
            hash,
            role as Role,
            input.organization_id
        ).fetch_one(db_pool).await?;

        Ok(record)
    }

    pub async fn create_with_profile(
        db_pool: &PgPool,
        input: &CreateUserWithProfileInput,
    ) -> Result<Self, Error> {
        let hash = bcrypt::hash(&input.password).unwrap();

        // Very cumbersome to get PostGIS geography type to insert with sqlx
        let coordinates: geo_types::Geometry<f64> = input
            .address
            .coordinates
            .as_ref()
            .map(|c| geo::Point::new(c.latitude, c.longitude))
            .unwrap()
            .into();

        let record = sqlx::query_as!(
            User,
            r#"
                WITH ins_user AS (
                    INSERT INTO populist_user (email, username, password, role, confirmation_token)
                    VALUES (LOWER($1), LOWER($2), $3, $4, $16)
                    RETURNING id, email, username, password, role AS "role:Role", organization_id, created_at, confirmed_at, updated_at
                ),
                ins_address AS (
                    INSERT INTO address (line_1, line_2, city, state, county, country, postal_code, geog, congressional_district, state_senate_district, state_house_district)
                    VALUES ($5, $6, $7, $8, $9, $10, $11, $12::geography, $13, $14, $15)
                    RETURNING id
                ),
                ins_profile AS (
                    INSERT INTO user_profile (address_id, user_id)
                    VALUES ((SELECT id FROM ins_address), (SELECT id FROM ins_user))
                )
                SELECT ins_user.* FROM ins_user
            "#,
            input.email,
            input.username,
            hash,
            Role::BASIC as Role,
            input.address.line_1,
            input.address.line_2,
            input.address.city,
            input.address.state.to_string(),
            input.address.county,
            input.address.country,
            input.address.postal_code,
            wkb::geom_to_wkb(&coordinates).unwrap() as _,
            input.address.congressional_district,
            input.address.state_senate_district,
            input.address.state_house_district,
            input.confirmation_token,
        ).fetch_one(db_pool).await?;

        // Need to handle case of existing user

        Ok(record)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, username, password, role AS "role:Role", organization_id, created_at, confirmed_at, updated_at FROM populist_user 
                WHERE $1 = id;
            "#,
            id
        ).fetch_optional(db_pool).await?;

        match record {
            Some(record) => Ok(record),
            None => Err(Error::EmailOrUsernameNotFound),
        }
    }

    pub async fn find_by_email_or_username(
        db_pool: &PgPool,
        email_or_username: String,
    ) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            User,
            r#"
                SELECT 
                    id, 
                    email, 
                    username, 
                    password, 
                    role AS "role:Role", 
                    organization_id,
                    created_at, 
                    confirmed_at, 
                    updated_at 
                FROM populist_user 
                WHERE LOWER($1) IN(email, username);
            "#,
            email_or_username
        )
        .fetch_optional(db_pool)
        .await?;

        match record {
            Some(record) => Ok(record),
            None => Err(Error::EmailOrUsernameNotFound),
        }
    }

    pub async fn validate_email_exists(db_pool: &PgPool, email: String) -> Result<bool, Error> {
        let existing_user = sqlx::query!(
            r#"
            SELECT id FROM populist_user WHERE email = LOWER($1)
        "#,
            email
        )
        .fetch_optional(db_pool)
        .await?;

        if let Some(_user) = existing_user {
            Ok(true)
        } else {
            Err(Error::EmailOrUsernameNotFound)
        }
    }

    pub async fn set_last_login_at(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            User,
            r#"
                UPDATE populist_user
                SET last_login_at = now()
                WHERE id = $1
                RETURNING id, email, username, password, role AS "role:Role", organization_id, created_at, confirmed_at, updated_at
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn reset_password(
        db_pool: &PgPool,
        new_password: String,
        reset_token: String,
    ) -> Result<Self, Error> {
        let hash = bcrypt::hash(&new_password).unwrap();

        let update_result = sqlx::query_as!(User,
            r#"
                UPDATE populist_user
                SET password = $1,
                    reset_token = NULL
                WHERE reset_token = $2
                AND reset_token_expires_at > now()
                RETURNING id, email, username, password, role AS "role:Role", organization_id, created_at, confirmed_at, updated_at
            "#,
            hash,
            reset_token
        )
        .fetch_optional(db_pool)
        .await;

        if let Ok(Some(user)) = update_result {
            Ok(user)
        } else {
            Err(Error::ResetTokenInvalid)
        }
    }

    pub async fn update_password(
        db_pool: &PgPool,
        new_password: String,
        user_id: uuid::Uuid,
    ) -> Result<bool, Error> {
        let hash = bcrypt::hash(&new_password).unwrap();

        let update_result = sqlx::query!(
            r#"
                UPDATE populist_user
                SET password = $1
                WHERE id = $2
            "#,
            hash,
            user_id
        )
        .execute(db_pool)
        .await;

        if update_result.is_ok() {
            Ok(true)
        } else {
            Err(Error::Custom(
                "Your password could not be updated".to_string(),
            ))
        }
    }

    pub async fn update_address(
        db_pool: &PgPool,
        user_id: uuid::Uuid,
        address: AddressInput,
    ) -> Result<Address, Error> {
        let geocodio = GeocodioProxy::new().unwrap();
        let address_clone = address.clone();
        let geocode_result = geocodio
            .geocode(
                geocodio::AddressParams::AddressInput(geocodio::AddressInput {
                    line_1: address_clone.line_1,
                    line_2: address_clone.line_2,
                    city: address_clone.city,
                    state: address_clone.state.to_string(),
                    country: address_clone.country,
                    postal_code: address_clone.postal_code,
                }),
                Some(&["cd", "stateleg"]),
            )
            .await;

        match geocode_result {
            Ok(geocodio_data) => {
                let coordinates = geocodio_data.results[0].location.clone();
                let county = geocodio_data.results[0].address_components.county.clone();
                let primary_result = geocodio_data.results[0].fields.as_ref().unwrap();
                let congressional_district =
                    primary_result.congressional_districts.as_ref().unwrap()[0]
                        .district_number
                        .to_string();
                let state_legislative_districts =
                    primary_result.state_legislative_districts.as_ref().unwrap();
                let state_house_district = &state_legislative_districts.house[0].district_number;
                let state_senate_district = &state_legislative_districts.senate[0].district_number;
                let lat = coordinates.latitude;
                let lon = coordinates.longitude;
                let coordinates: geo_types::Geometry<f64> = Some(coordinates)
                    .as_ref()
                    .map(|c| geo::Point::new(c.latitude, c.longitude))
                    .unwrap()
                    .into();

                let updated_record_result = sqlx::query_as!(
                    Address,
                    r#"
                   UPDATE
                        address a
                    SET
                        line_1 = $2,
                        line_2 = $3,
                        city = $4,
                        state = $5,
                        county = $6,
                        postal_code = $7,
                        country = $8,
                        geog = $9::geography,
                        congressional_district = $10,
                        state_house_district = $11,
                        state_senate_district = $12,
                        geom = ST_GeomFromText($13, 4326),
                        lat = $14,
                        lon = $15
                    FROM
                        user_profile up
                    WHERE
                        up.address_id = a.id
                        AND up.user_id = $1
                    RETURNING
                        a.id,
                        line_1,
                        line_2,
                        city,
                        state AS "state:State",
                        postal_code,
                        country,
                        county,
                        congressional_district,
                        state_senate_district,
                        state_house_district
                "#,
                    user_id,
                    address.line_1,
                    address.line_2,
                    address.city,
                    address.state as State,
                    county,
                    address.postal_code,
                    address.country,
                    wkb::geom_to_wkb(&coordinates).unwrap() as _,
                    Some(congressional_district),
                    state_house_district,
                    state_senate_district,
                    format!("POINT({} {})", lon, lat), // A string we pass into ST_GeomFromText function
                    lat,
                    lon,
                )
                .fetch_one(db_pool)
                .await;

                match updated_record_result {
                    Ok(updated_record) => Ok(updated_record),
                    Err(err) => Err(err.into()),
                }
            }
            Err(_) => Err(Error::Custom(
                "This is not a valid voting address".to_string(),
            )),
        }
    }
}
