use async_graphql::InputObject;
use sqlx::postgres::PgPool;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Party {
    pub id: uuid::Uuid,
    pub slug: String,
    pub name: String,
    pub fec_code: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(InputObject, Default, Debug)]
pub struct UpsertPartyInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub name: Option<String>,
    pub fec_code: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

impl Party {
    pub async fn upsert_from_source(
        db_pool: &PgPool,
        input: &UpsertPartyInput,
    ) -> Result<Self, sqlx::Error> {
        input
            .slug
            .as_ref()
            .ok_or("slug is required")
            .map_err(|err| sqlx::Error::AnyDriverError(err.into()))?;

        sqlx::query_as!(
            Party,
            r#"
                INSERT INTO party
                (slug, name, description, fec_code, notes)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (slug) DO UPDATE SET
                    name = COALESCE($2, party.name),
                    description = COALESCE($3, party.description),
                    fec_code = COALESCE($4, party.fec_code),
                    notes = COALESCE($5, party.notes)
                RETURNING id, slug, name, description, fec_code, notes
            "#,
            input.slug,
            input.name,
            input.description,
            input.fec_code,
            input.notes,
        )
        .fetch_one(db_pool)
        .await
    }
}
