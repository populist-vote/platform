use crate::{
    models::enums::{PoliticalParty, State},
    CreateOrConnectIssueTagInput, CreateOrConnectOrganizationInput, DateTime, IssueTag,
    Organization, OrganizationIdentifier,
};
use async_graphql::InputObject;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use slugify::slugify;
use sqlx::{postgres::PgPool, FromRow};

#[derive(FromRow, Debug, Clone)]
pub struct Politician {
    pub id: uuid::Uuid,
    pub slug: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub date_of_birth: Option<NaiveDate>,
    pub office_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub party: Option<PoliticalParty>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Value,
    pub votesmart_candidate_ratings: Value,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub upcoming_race_id: Option<uuid::Uuid>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject, Default, Debug, Serialize, Deserialize)]
pub struct CreatePoliticianInput {
    pub slug: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub date_of_birth: Option<NaiveDate>,
    pub office_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub party: Option<PoliticalParty>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
    pub organization_endorsements: Option<CreateOrConnectOrganizationInput>,
    pub politician_endorsements: Option<CreateOrConnectPoliticianInput>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<Value>,
    pub votesmart_candidate_ratings: Option<Value>,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub upcoming_race_id: Option<uuid::Uuid>,
}

#[derive(InputObject, Default, Serialize, Deserialize)]
pub struct UpdatePoliticianInput {
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub date_of_birth: Option<NaiveDate>,
    pub office_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub party: Option<PoliticalParty>,
    pub issue_tags: Option<CreateOrConnectIssueTagInput>,
    pub organization_endorsements: Option<CreateOrConnectOrganizationInput>,
    pub politician_endorsements: Option<CreateOrConnectPoliticianInput>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<Value>,
    pub votesmart_candidate_ratings: Option<Value>,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub upcoming_race_id: Option<uuid::Uuid>,
}

pub enum PoliticianIdentifier {
    Uuid(uuid::Uuid),
    Slug(String),
}

#[derive(Debug, Serialize, Deserialize, InputObject)]
pub struct CreateOrConnectPoliticianInput {
    pub create: Option<Vec<CreatePoliticianInput>>,
    pub connect: Option<Vec<String>>, // Accept UUIDs or slugs
}

#[derive(InputObject, Default, Debug)]
pub struct PoliticianSearch {
    home_state: Option<State>,
    name: Option<String>,
    party: Option<PoliticalParty>,
}

impl CreatePoliticianInput {
    fn full_name(&self) -> String {
        format!(
            "{first_name} {last_name} {suffix}",
            first_name = &self.preferred_name.as_ref().unwrap_or(&self.first_name),
            last_name = &self.last_name,
            suffix = &self.suffix.as_ref().unwrap_or(&"".to_string())
        )
        .trim_end()
        .to_string()
    }
}

impl Politician {
    pub async fn create(
        db_pool: &PgPool,
        input: &CreatePoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        let slug = slugify!(&CreatePoliticianInput::full_name(input));

        let vs_candidate_bio = match input.votesmart_candidate_bio.to_owned() {
            Some(bio) => bio,
            None => json!({}),
        };

        let vs_candidate_ratings = match input.votesmart_candidate_ratings.to_owned() {
            Some(ratings) => ratings,
            None => json!([]),
        };

        let record = sqlx::query_as!(
            Politician,
            r#"
                WITH ins_author AS (
                INSERT INTO author (author_type)
                        VALUES('politician') ON CONFLICT DO NOTHING
                    RETURNING
                        id AS author_id
                ),
                p AS (
                INSERT INTO politician (id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state,
                        date_of_birth,
                        office_id,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party,
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id)
                        VALUES(
                            (SELECT author_id FROM ins_author),
                            $1,
                            $2,
                            $3,
                            $4,
                            $5,
                            $6,
                            $7,
                            $8,
                            $9,
                            $10,
                            $11,
                            $12,
                            $13,
                            $14,
                            $15,
                            $16,
                            $17,
                            $18,
                            $19,
                            $20,
                            $21,
                            $22,
                            $23,
                            $24, 
                            $25,
                            $26,
                            $27,
                            $28)
                    RETURNING
                        id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        created_at,
                        updated_at
                )
                SELECT
                    p.*
                FROM
                    p
            "#,
            slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.suffix,
            input.preferred_name,
            input.biography,
            input.biography_source,
            input.home_state as Option<State>,
            input.date_of_birth as Option<NaiveDate>,
            input.office_id,
            input.website_url,
            input.campaign_website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.youtube_url,
            input.linkedin_url,
            input.tiktok_url,
            input.email,
            input.party as Option<PoliticalParty>,
            input.votesmart_candidate_id,
            vs_candidate_bio,
            vs_candidate_ratings,
            input.legiscan_people_id,
            input.crp_candidate_id,
            input.fec_candidate_id,
            input.upcoming_race_id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn create_optional(
        db_pool: &PgPool,
        input: &CreatePoliticianInput,
    ) -> Result<Option<Self>, sqlx::Error> {
        let slug = slugify!(&CreatePoliticianInput::full_name(input));

        let vs_candidate_bio = match input.votesmart_candidate_bio.to_owned() {
            Some(bio) => bio,
            None => json!({}),
        };

        let vs_candidate_ratings = match input.votesmart_candidate_ratings.to_owned() {
            Some(ratings) => ratings,
            None => json!([]),
        };

        let record = sqlx::query_as!(
            Politician,
            r#"
                WITH ins_author AS (
                    INSERT INTO author (author_type) VALUES ('politician')
                    ON CONFLICT DO NOTHING
                    RETURNING id AS author_id
                ),
                p AS (
                    INSERT INTO politician (id, slug, first_name, middle_name, last_name, home_state, date_of_birth, office_id, party, votesmart_candidate_id, votesmart_candidate_bio, votesmart_candidate_ratings, website_url, upcoming_race_id) 
                    VALUES ((SELECT author_id FROM ins_author), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                    RETURNING
                        id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        created_at,
                        updated_at
                )
                SELECT p.* FROM p
            "#,
            slug,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.home_state as Option<State>,
            input.date_of_birth as Option<NaiveDate>,
            input.office_id,
            input.party as Option<PoliticalParty>,
            input.votesmart_candidate_id,
            vs_candidate_bio,
            vs_candidate_ratings,
            input.website_url,
            input.upcoming_race_id
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn update(
        db_pool: &PgPool,
        id: Option<uuid::Uuid>,
        votesmart_candidate_id: Option<i32>,
        input: &UpdatePoliticianInput,
    ) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
                UPDATE politician 
                SET first_name = COALESCE($3, first_name),
                    middle_name = COALESCE($4, middle_name),
                    last_name = COALESCE($5, last_name),
                    suffix = COALESCE($6, suffix),
                    preferred_name = COALESCE($7, preferred_name),
                    biography = COALESCE($8, biography),
                    biography_source = COALESCE($9, biography_source),
                    home_state= COALESCE($10, home_state),
                    date_of_birth = COALESCE($11, date_of_birth),
                    office_id = COALESCE($12, office_id),
                    thumbnail_image_url = COALESCE($13, thumbnail_image_url),
                    website_url = COALESCE($14, website_url),
                    campaign_website_url = COALESCE($15, campaign_website_url),
                    facebook_url = COALESCE($16, facebook_url),
                    twitter_url = COALESCE($17, twitter_url),
                    instagram_url = COALESCE($18, instagram_url),
                    youtube_url = COALESCE($19, youtube_url),
                    linkedin_url = COALESCE($20, linkedin_url),
                    tiktok_url = COALESCE($21, tiktok_url),
                    email = COALESCE($22, email),
                    party = COALESCE($23, party),
                    votesmart_candidate_id = COALESCE($2, votesmart_candidate_id),
                    votesmart_candidate_bio = COALESCE($24, votesmart_candidate_bio),
                    votesmart_candidate_ratings = COALESCE($25, votesmart_candidate_ratings),
                    legiscan_people_id = COALESCE($26, legiscan_people_id),
                    crp_candidate_id = COALESCE($27, crp_candidate_id),
                    fec_candidate_id = COALESCE($28, fec_candidate_id),
                    upcoming_race_id = COALESCE($29, upcoming_race_id)
                WHERE id=$1
                OR votesmart_candidate_id = $2
                RETURNING
                        id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        created_at,
                        updated_at
            "#,
            id,
            votesmart_candidate_id,
            input.first_name,
            input.middle_name,
            input.last_name,
            input.suffix,
            input.preferred_name,
            input.biography,
            input.biography_source,
            input.home_state as Option<State>,
            input.date_of_birth as Option<NaiveDate>,
            input.office_id,
            input.thumbnail_image_url,
            input.website_url,
            input.campaign_website_url,
            input.facebook_url,
            input.twitter_url,
            input.instagram_url,
            input.youtube_url,
            input.linkedin_url,
            input.tiktok_url,
            input.email,
            input.party as Option<PoliticalParty>,
            input.votesmart_candidate_bio,
            input.votesmart_candidate_ratings,
            input.legiscan_people_id,
            input.crp_candidate_id,
            input.fec_candidate_id,
            input.upcoming_race_id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM politician WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn index(db_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let records = sqlx::query_as!(
            Politician,
            r#"SELECT id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        created_at,
                        updated_at FROM politician"#,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        created_at,
                        updated_at FROM politician
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(record)
    }

    pub async fn find_by_slug(db_pool: &PgPool, slug: String) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        created_at,
                        updated_at FROM politician
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn search(
        db_pool: &PgPool,
        search: &PoliticianSearch,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let search_query =
            crate::process_search_query(search.name.to_owned().unwrap_or_else(|| "".to_string()));

        let records = sqlx::query_as!(
            Politician,
            r#"
                SELECT id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        created_at,
                        updated_at FROM politician
                WHERE (($1::text = '') IS NOT FALSE OR to_tsvector('simple', concat_ws(' ', first_name, middle_name, last_name, preferred_name)) @@ to_tsquery('simple', $1))
                AND ($2::state IS NULL OR home_state = $2)
                AND ($3::political_party IS NULL OR party = $3)
                ORDER BY last_name ASC
            "#,
            search_query,
            search.home_state as Option<State>,
            search.party as Option<PoliticalParty>,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn organization_endorsements(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<Organization>, sqlx::Error> {
        let records = sqlx::query_as!(Organization,
            r#"
                SELECT o.id, slug, name, description, thumbnail_image_url, website_url, facebook_url, twitter_url, instagram_url, email, votesmart_sig_id, headquarters_address_id, headquarters_phone, tax_classification, o.created_at, o.updated_at  FROM organization o
                JOIN politician_organization_endorsements
                ON politician_organization_endorsements.organization_id = o.id
                WHERE politician_organization_endorsements.politician_id = $1
            "#, 
        politician_id).fetch_all(db_pool).await?;

        Ok(records)
    }

    pub async fn politician_endorsements(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<Politician>, sqlx::Error> {
        let records = sqlx::query_as!(
            Politician,
            r#"
                SELECT p.id,
                        slug,
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        preferred_name,
                        biography,
                        biography_source,
                        home_state AS "home_state:State",
                        date_of_birth,
                        office_id,
                        thumbnail_image_url,
                        website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        upcoming_race_id,
                        p.created_at,
                        p.updated_at FROM politician p
                JOIN politician_politician_endorsements
                ON politician_politician_endorsements.politician_endorsement_id = p.id
                WHERE politician_politician_endorsements.politician_id = $1
            "#,
            politician_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn connect_organization(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
        organization_identifier: OrganizationIdentifier,
    ) -> Result<(), sqlx::Error> {
        match organization_identifier {
            OrganizationIdentifier::Uuid(organization_id) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_organization_endorsements (politician_id, organization_id)
                        VALUES ($1, $2)
                    "#,
                    politician_id,
                    organization_id
                )
                .execute(db_pool)
                .await?;
            }
            OrganizationIdentifier::Slug(organization_slug) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_organization_endorsements (politician_id, organization_id)
                        VALUES ($1, (SELECT id FROM organization WHERE slug = $2))
                    "#,
                    politician_id,
                    organization_slug
                )
                .execute(db_pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn connect_politician(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
        politician_identifier: PoliticianIdentifier,
    ) -> Result<(), sqlx::Error> {
        match politician_identifier {
            PoliticianIdentifier::Uuid(politician_endorsement_id) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_politician_endorsements (politician_id, politician_endorsement_id)
                        VALUES ($1, $2)
                    "#,
                    politician_id,
                    politician_endorsement_id
                )
                .execute(db_pool)
                .await?;
            }
            PoliticianIdentifier::Slug(politician_endorsement_slug) => {
                sqlx::query_as!(
                    Politician,
                    r#"
                        INSERT INTO politician_politician_endorsements (politician_id, politician_endorsement_id)
                        VALUES ($1, (SELECT id FROM politician WHERE slug = $2))
                    "#,
                    politician_id,
                    politician_endorsement_slug
                )
                .execute(db_pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn connect_issue_tag(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
        issue_tag_id: uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Politician,
            r#"
                INSERT INTO politician_issue_tags (politician_id, issue_tag_id) 
                VALUES ($1, $2)
            "#,
            politician_id,
            issue_tag_id
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        politician_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN politician_issue_tags
                ON politician_issue_tags.issue_tag_id = it.id
                WHERE politician_issue_tags.politician_id = $1
            "#,
            politician_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }
}
