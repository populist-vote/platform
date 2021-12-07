use sqlx::PgPool;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, sqlx::Type)]
pub enum VoteDirection {
    UP = 1,
    DOWN = -1,
    UNVOTE = 0,
}

#[derive(Display, Debug, Clone, sqlx::Type, EnumString)]
pub enum VotableType {
    Argument,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Vote {
    pub populist_user_id: uuid::Uuid,
    pub votable_id: uuid::Uuid,
    pub votable_type: VotableType,
    pub direction: VoteDirection,
}

impl Vote {
    // Generic upvote/downvote function that can be applied to any populist object
    pub async fn upsert(db_pool: &PgPool, vote: Vote) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO vote (populist_user_id, votable_id, votable_type, direction)
                VALUES ($1, $2, $3, $4) 
                ON CONFLICT ON CONSTRAINT unique_person_per_votable DO 
                UPDATE SET direction = $4
            "#,
            vote.populist_user_id,
            vote.votable_id,
            vote.votable_type.to_string(),
            vote.direction as i32,
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(())
    }

    pub async fn count(db_pool: &PgPool, votable_id: uuid::Uuid) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!(
            r#"
                SELECT SUM (direction) AS total
                FROM vote
                WHERE votable_id = $1
            "#,
            votable_id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(record.total.unwrap_or(0))
    }
}
