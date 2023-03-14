use crate::{
    models::enums::{ArgumentPosition, AuthorType, BillStatus},
    Argument, Chamber, CreateArgumentInput, DateTime, IssueTag, Politician,
};
use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::NaiveDate;
use serde_json::Value as JSON;
use slugify::slugify;
use sqlx::{postgres::PgPool, FromRow};

use super::enums::{PoliticalParty, PoliticalScope, State};

#[derive(FromRow, Debug, Clone)]
pub struct Bill {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub bill_number: String,
    pub status: BillStatus,
    pub description: Option<String>,
    pub session_id: Option<uuid::Uuid>,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub votesmart_bill_id: Option<i32>,
    pub legiscan_bill_id: Option<i32>,
    pub legiscan_committee: Option<String>,
    pub legiscan_last_action: Option<String>,
    pub legiscan_last_action_date: Option<NaiveDate>,
    pub legiscan_data: JSON,
    pub history: JSON,
    pub state: Option<State>,
    pub political_scope: PoliticalScope,
    pub bill_type: String,
    pub chamber: Option<Chamber>,
    pub attributes: JSON,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(InputObject)]
pub struct UpsertBillInput {
    pub id: Option<uuid::Uuid>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub bill_number: String,
    pub status: BillStatus,
    pub description: Option<String>,
    pub session_id: uuid::Uuid,
    pub official_summary: Option<String>,
    pub populist_summary: Option<String>,
    pub full_text_url: Option<String>,
    pub legiscan_bill_id: Option<i32>,
    pub legiscan_session_id: Option<i32>,
    pub legiscan_committee_id: Option<i32>,
    pub legiscan_committee: Option<String>,
    pub legiscan_last_action: Option<String>,
    pub legiscan_last_action_date: Option<NaiveDate>,
    pub history: Option<JSON>,
    pub state: Option<State>,
    pub legiscan_data: Option<JSON>,
    pub votesmart_bill_id: Option<i32>,
    pub arguments: Option<Vec<CreateArgumentInput>>,
    pub political_scope: Option<PoliticalScope>,
    pub bill_type: Option<String>,
    pub chamber: Option<Chamber>,
    pub attributes: Option<JSON>,
}

#[derive(InputObject, Default, Debug)]
pub struct BillFilter {
    query: Option<String>,
    political_scope: Option<PoliticalScope>,
    state: Option<State>,
    year: Option<i32>,
    status: Option<BillStatus>,
}

#[derive(InputObject, Default, Debug)]
pub struct BillSort {
    popularity: Option<PopularitySort>,
}

#[derive(SimpleObject)]
pub struct PublicVotes {
    pub support: Option<i64>,
    pub neutral: Option<i64>,
    pub oppose: Option<i64>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum PopularitySort {
    #[default]
    MostPopular,
    MostSupported,
    MostOpposed,
}

impl Bill {
    pub async fn upsert(db_pool: &PgPool, input: &UpsertBillInput) -> Result<Self, sqlx::Error> {
        let id = input.id.unwrap_or_else(uuid::Uuid::new_v4);
        let slug = input.slug.clone().unwrap_or_else(|| {
            slugify!(&input
                .title
                .clone()
                .unwrap_or_else(|| input.bill_number.clone()))
        });
        let title = input
            .title
            .clone()
            .unwrap_or_else(|| input.bill_number.clone());
        let legiscan_data = input
            .legiscan_data
            .clone()
            .unwrap_or_else(|| serde_json::from_str("{}").unwrap());

        let record = sqlx::query_as!(
            Bill,
            r#"
            INSERT INTO bill (
                id,
                slug,
                title,
                bill_number,
                status,
                description,
                session_id,
                official_summary,
                populist_summary,
                full_text_url,
                legiscan_bill_id,
                legiscan_committee,
                legiscan_last_action,
                legiscan_last_action_date,
                legiscan_data,
                votesmart_bill_id,
                history,
                state,
                political_scope,
                bill_type,
                chamber,
                attributes
            ) VALUES (
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
                $22
            ) ON CONFLICT (id) DO UPDATE 
            SET
                slug = COALESCE($2, bill.slug),
                title = COALESCE($3, bill.title),
                bill_number = COALESCE($4, bill.bill_number),
                status = COALESCE($5, bill.status),
                description = COALESCE($6, bill.description),
                session_id = COALESCE($7, bill.session_id),
                official_summary = COALESCE($8, bill.official_summary),
                populist_summary = COALESCE($9, bill.populist_summary),
                full_text_url = COALESCE($10, bill.full_text_url),
                legiscan_bill_id = COALESCE($11, bill.legiscan_bill_id),
                legiscan_committee = COALESCE($12, bill.legiscan_committee),
                legiscan_last_action = COALESCE($13, bill.legiscan_last_action),
                legiscan_last_action_date = COALESCE($14, bill.legiscan_last_action_date),
                legiscan_data = COALESCE($15, bill.legiscan_data),
                votesmart_bill_id = COALESCE($16, bill.votesmart_bill_id),
                history = COALESCE($17, bill.history),
                state = COALESCE($18, bill.state),
                political_scope = COALESCE($19, bill.political_scope),
                bill_type = COALESCE($20, bill.bill_type),
                chamber = COALESCE($21, bill.chamber),
                attributes = COALESCE($22, bill.attributes)
            RETURNING 
                id,
                slug,
                title,
                bill_number,
                status AS "status: BillStatus",
                description,
                session_id,
                official_summary,
                populist_summary,
                full_text_url,
                legiscan_bill_id,
                legiscan_committee,
                legiscan_last_action,
                legiscan_last_action_date,
                legiscan_data,
                votesmart_bill_id,
                history,
                state AS "state: State",
                political_scope AS "political_scope: PoliticalScope",
                bill_type,
                chamber AS "chamber: Chamber",
                attributes,
                created_at,
                updated_at
            "#,
            id,
            slug,
            title,
            input.bill_number,
            input.status as BillStatus,
            input.description,
            input.session_id,
            input.official_summary,
            input.populist_summary,
            input.full_text_url,
            input.legiscan_bill_id,
            input.legiscan_committee,
            input.legiscan_last_action,
            input.legiscan_last_action_date as Option<NaiveDate>,
            legiscan_data as JSON,
            input.votesmart_bill_id,
            input.history.clone() as Option<JSON>,
            input.state as Option<State>,
            input.political_scope as Option<PoliticalScope>,
            input.bill_type,
            input.chamber as Option<Chamber>,
            input.attributes.clone() as Option<JSON>,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete(db_pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM bill WHERE id=$1", id)
            .execute(db_pool)
            .await?;
        Ok(())
    }

    pub async fn filter(
        db_pool: &PgPool,
        filter: &BillFilter,
        sort: &BillSort,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let search_query = filter.query.to_owned().unwrap_or_default();

        let query = format!(
            r#"
            SELECT
                id,
                slug,
                title,
                bill_number,
                status,
                description,
                session_id,
                official_summary,
                populist_summary,
                full_text_url,
                legiscan_bill_id,
                legiscan_committee,
                legiscan_last_action,
                legiscan_last_action_date,
                legiscan_data,
                history,
                state,
                votesmart_bill_id,
                political_scope,
                bill_type,
                chamber,
                bill.attributes,
                bill.created_at,
                bill.updated_at,
                rank_bill_number,
                rank_title,
                rank_description
            FROM
                bill
                LEFT JOIN bill_public_votes bpv ON bill.id = bpv.bill_id,
                to_tsvector(bill_number || ' ' || title || ' ' || COALESCE(description, '')) document,
                websearch_to_tsquery($1::text) query,
                NULLIF(ts_rank(to_tsvector(bill_number), query), 0) rank_bill_number,
                NULLIF(ts_rank(to_tsvector(title), query), 0) rank_title,
                NULLIF(ts_rank(to_tsvector(description), query), 0) rank_description
            WHERE
                document @@ query
                AND($2::bill_status IS NULL
                    OR status = $2)
                AND($3::political_scope IS NULL
                    OR political_scope = $3)
                AND(($4::state IS NULL
                    OR state = $4)
                OR $3::political_scope = 'federal')
            GROUP BY (bill.id, rank_bill_number, rank_title, rank_description)
            ORDER BY {order_by}
            LIMIT 20
        "#,
            order_by = match sort.popularity {
                Some(PopularitySort::MostPopular) => "rank_bill_number, rank_title, rank_description, COUNT(bpv.*) DESC NULLS LAST",
                Some(PopularitySort::MostSupported) =>
                    "rank_bill_number, rank_title, rank_description, SUM(CASE WHEN bpv.position = 'support' THEN 1 ELSE 0 END)DESC NULLS LAST",
                Some(PopularitySort::MostOpposed) =>
                    "rank_bill_number, rank_title, rank_description, SUM(CASE WHEN bpv.position = 'oppose' THEN 1 ELSE 0 END) DESC NULLS LAST",
                None => "rank_bill_number, rank_title, rank_description DESC NULLS LAST",
            }
        );

        let records = sqlx::query_as::<_, Bill>(&query)
            .bind(search_query)
            .bind(filter.status as Option<BillStatus>)
            .bind(filter.political_scope)
            .bind(filter.state)
            .fetch_all(db_pool)
            .await?;
        Ok(records)
    }

    pub async fn popular(db_pool: &PgPool, filter: &BillFilter) -> Result<Vec<Self>, sqlx::Error> {
        let search_query = filter.query.to_owned().unwrap_or_default();

        let records = sqlx::query_as!(
            Bill,
            r#"
                SELECT id, slug, title, bill_number, status AS "status: BillStatus", description, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_committee, legiscan_last_action, legiscan_last_action_date, legiscan_data, history, state AS "state: State", votesmart_bill_id, political_scope AS "political_scope: PoliticalScope", bill_type, chamber AS "chamber: Chamber", bill.attributes, session_id, bill.created_at, bill.updated_at FROM bill
                JOIN bill_public_votes bpv ON bill.id = bpv.bill_id
                WHERE (($1::text = '') IS NOT FALSE OR to_tsvector('simple', concat_ws(' ', title, description, bill_number)) @@ to_tsquery('simple', $1))
                AND ($2::bill_status IS NULL OR status = $2)
                AND ($3::political_scope IS NULL OR political_scope = $3)
                AND (($4::state IS NULL OR state = $4) OR $3::political_scope = 'federal')
                GROUP BY (bill.id)
                ORDER BY COUNT(bill.id) DESC
                LIMIT 20
            "#,
            search_query,
            filter.status as Option<BillStatus>,
            filter.political_scope as Option<PoliticalScope>,
            filter.state as Option<State>,
        )
        .fetch_all(db_pool)
        .await?;
        Ok(records)
    }

    pub async fn find_by_id(db_pool: &PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Bill,
            r#"
                SELECT id, slug, title, bill_number, status AS "status: BillStatus", description, session_id, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_committee, legiscan_last_action, legiscan_last_action_date, legiscan_data, history, state AS "state: State", votesmart_bill_id, political_scope AS "political_scope: PoliticalScope", bill_type, chamber AS "chamber: Chamber", attributes, created_at, updated_at FROM bill 
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(record)
    }

    pub async fn find_by_slug(db_pool: &PgPool, slug: &str) -> Result<Self, sqlx::Error> {
        let record = sqlx::query_as!(
            Bill,
            r#"
                SELECT id, slug, title, bill_number, status AS "status: BillStatus", description, session_id, official_summary, populist_summary, full_text_url, legiscan_bill_id, legiscan_committee, legiscan_last_action, legiscan_last_action_date, legiscan_data, history, state AS "state: State", votesmart_bill_id, political_scope AS "political_scope: PoliticalScope", bill_type, chamber AS "chamber: Chamber", attributes, created_at, updated_at FROM bill
                WHERE slug = $1
            "#,
            slug
        )
        .fetch_one(db_pool)
        .await?;
        Ok(record)
    }

    pub async fn issue_tags(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
    ) -> Result<Vec<IssueTag>, sqlx::Error> {
        let records = sqlx::query_as!(IssueTag,
            r#"
                SELECT it.id, slug, name, description, category, it.created_at, it.updated_at FROM issue_tag it
                JOIN bill_issue_tags
                ON bill_issue_tags.issue_tag_id = it.id
                WHERE bill_issue_tags.bill_id = $1
            "#,
            bill_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn sponsors(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
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
                        upcoming_race_id,
                        thumbnail_image_url,
                        assets,
                        official_website_url,
                        campaign_website_url,
                        facebook_url,
                        twitter_url,
                        instagram_url,
                        youtube_url,
                        linkedin_url,
                        tiktok_url,
                        email,
                        phone,
                        party AS "party:PoliticalParty",
                        votesmart_candidate_id,
                        votesmart_candidate_bio,
                        votesmart_candidate_ratings,
                        legiscan_people_id,
                        crp_candidate_id,
                        fec_candidate_id,
                        race_wins,
                        race_losses,
                        p.created_at,
                        p.updated_at FROM politician p 
                JOIN bill_sponsors bs ON bs.politician_id = p.id 
                WHERE bs.bill_id = $1
            "#,
            bill_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(records)
    }

    pub async fn create_bill_argument(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
        author_id: uuid::Uuid,
        input: &CreateArgumentInput,
    ) -> Result<Argument, sqlx::Error> {
        let record = sqlx::query_as_unchecked!(
            Argument,
            r#"
                WITH ins_argument AS (
                    INSERT INTO argument (author_id, title, position, body) 
                    VALUES ($2, $3, $4, $5) 
                    RETURNING id, author_id, title, position, body, created_at, updated_at
                ),
                ins_bill_argument AS (
                    INSERT INTO bill_arguments (bill_id, argument_id) 
                    VALUES ($1, (SELECT id FROM ins_argument))
                )
                SELECT ins_argument.id, ins_argument.author_id, a.author_type AS "author_type:AuthorType", ins_argument.title, ins_argument.position AS "position:ArgumentPosition", ins_argument.body, ins_argument.created_at, ins_argument.updated_at
                FROM ins_argument JOIN author AS a ON a.id = ins_argument.author_id
            "#,
            bill_id,
            author_id,
            input.title,
            input.position as ArgumentPosition,
            input.body,
        ).fetch_one(db_pool).await?;

        Ok(record)
    }

    pub async fn arguments(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
    ) -> Result<Vec<Argument>, sqlx::Error> {
        let records = sqlx::query_as!(Argument,
            r#"
                SELECT arg.id, arg.author_id, author.author_type AS "author_type:AuthorType", title, position AS "position:ArgumentPosition", body, arg.created_at, arg.updated_at 
                FROM argument AS arg
                JOIN author ON author.id = arg.author_id
                JOIN bill_arguments ON bill_arguments.argument_id = arg.id
                WHERE bill_arguments.bill_id = $1
            "#,
            bill_id
        ).fetch_all(db_pool).await?;

        Ok(records)
    }

    pub async fn connect_issue_tag(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
        issue_tag_id: uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Bill,
            r#"
                INSERT INTO bill_issue_tags (bill_id, issue_tag_id) 
                VALUES ($1, $2)
            "#,
            bill_id,
            issue_tag_id
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }

    pub async fn public_votes(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
    ) -> Result<PublicVotes, sqlx::Error> {
        let record = sqlx::query_as!(
            PublicVotes,
            r#"
                SELECT SUM(CASE WHEN position = 'support' THEN 1 ELSE 0 END) as support,
                       SUM(CASE WHEN position = 'neutral' THEN 1 ELSE 0 END) as neutral,
                       SUM(CASE WHEN position = 'oppose' THEN 1 ELSE 0 END) as oppose
                FROM bill_public_votes WHERE bill_id = $1
            "#,
            bill_id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record)
    }

    pub async fn upsert_public_vote(
        db_pool: &PgPool,
        bill_id: uuid::Uuid,
        user_id: uuid::Uuid,
        position: ArgumentPosition,
    ) -> Result<PublicVotes, sqlx::Error> {
        let _upsert = sqlx::query!(
            r#"
                INSERT INTO bill_public_votes (bill_id, user_id, position) 
                VALUES ($1, $2, $3) 
                ON CONFLICT (bill_id, user_id) DO UPDATE SET position = $3
            "#,
            bill_id,
            user_id,
            position as ArgumentPosition,
        )
        .execute(db_pool)
        .await?;

        let record = Bill::public_votes(db_pool, bill_id).await?;

        Ok(record)
    }
}
