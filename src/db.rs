use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    query, query_as, Pool,
};
use std::env;

use crate::models::*;

#[derive(Clone)]
pub struct Db {
    pool: Pool<Postgres>,
}

impl Db {
    pub async fn connect() -> Result<Self, anyhow::Error> {
        let db_url = env::var("DATABASE_URL")?;
        let pool = PgPoolOptions::new().connect(&db_url).await?;
        Ok(Self { pool })
    }


    pub async fn validate_token<T: AsRef<str>>(&self, token_id: T) -> Result<Token, anyhow::Error> {
        let token_id = token_id.as_ref();
        let token =
            query_as("SELECT * FROM tokens WHERE id = $1 AND expired_at > current_timestamp")
                .bind(token_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(token)
    }

    pub async fn boards(&self) -> Result<Vec<Board>, anyhow::Error> {
        let boards = query_as("SELECT * FROM boards")
            .fetch_all(&self.pool)
            .await?;
        Ok(boards)
    }

    pub async fn board_summary(&self, board_id: i64) -> Result<BoardSummary, anyhow::Error> {
        let counts: Vec<(i64, Status)> =
            query_as("SELECT count(*), status FROM cards WHERE board_id = $1 GROUP BY status")
                .bind(board_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(counts.into())
    }

    pub async fn create_board(&self, create_board: CreateBoard) -> Result<Board, anyhow::Error> {
        let board = query_as("INSERT INTO boards (name) VALUES ($1) RETURNING *")
            .bind(&create_board.name)
            .fetch_one(&self.pool)
            .await?;
        Ok(board)
    }

    pub async fn delete_board(&self, board_id: i64) -> Result<(), anyhow::Error> {
        query("DELETE FROM boards WHERE id = $1")
            .bind(board_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn cards(&self, board_id: i64) -> Result<Vec<Card>, anyhow::Error> {
        let cards = query_as("SELECT * FROM cards WHERE board_id = $1")
            .bind(board_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(cards)
    }

    pub async fn create_card(&self, create_card: CreateCard) -> Result<Card, anyhow::Error> {
        let card =
            query_as("INSERT INTO cards (board_id, description) VALUES ($1, $2) RETURNING *")
                .bind(&create_card.board_id)
                .bind(&create_card.description)
                .fetch_one(&self.pool)
                .await?;
        Ok(card)
    }

    pub async fn update_card(&self, card_id: i64, update_card: UpdateCard) -> Result<Card, anyhow::Error> {
        let card =
            query_as("UPDATE cards SET description = $1, status = $2 WHERE id = $3 RETURNING *")
                .bind(&update_card.description)
                .bind(&update_card.status)
                .bind(card_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(card)
    }

    pub async fn delete_card(&self, card_id: i64) -> Result<(), anyhow::Error> {
        query("DELETE FROM cards WHERE id = $1")
            .bind(card_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
