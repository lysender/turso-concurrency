use std::time::Duration;

use snafu::ResultExt;
use tokio::time::sleep;

use crate::Result;
use crate::db::db_pool::DbPool;
use crate::db::turso_decode::collect_row;
use crate::db::turso_params::{integer_param, new_query_params, text_param};
use crate::dto::PostDto;
use crate::error::{DbPrepareSnafu, DbStatementSnafu};

pub struct PooledPostRepo {
    db_pool: DbPool,
}

impl PooledPostRepo {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn get(&self, id: i64) -> Result<Option<PostDto>> {
        let query = r#"
            SELECT
                id,
                title,
                content
            FROM posts
            WHERE
                id = :id
            LIMIT 1
        "#;

        let mut q_params = new_query_params();
        q_params.push(integer_param(":id", id));

        let conn = self.db_pool.acquire().await?;
        let mut stmt = conn.prepare(query).await.context(DbPrepareSnafu)?;
        let row_result = stmt.query_row(q_params).await;
        let dto: Option<PostDto> = collect_row(row_result)?;
        Ok(dto)
    }

    pub async fn update(&self, id: i64, title: String, content: String) -> Result<bool> {
        let query = r#"
            UPDATE
                posts
            SET
                title = :title,
                content = :content
            WHERE
                id = :id
        "#;

        let mut q_params = new_query_params();
        q_params.push(integer_param(":id", id));
        q_params.push(text_param(":title", title));
        q_params.push(text_param(":content", content));

        let conn = self.db_pool.acquire().await?;
        let mut stmt = conn.prepare(query).await.context(DbPrepareSnafu)?;
        let affected = stmt.execute(q_params).await.context(DbStatementSnafu)?;

        Ok(affected > 0)
    }

    #[allow(dead_code)]
    pub async fn update_with_retry(&self, id: i64, title: String, content: String) -> Result<bool> {
        const MAX_ATTEMPTS: usize = 5;
        let mut backoff = Duration::from_millis(5);

        for attempt in 1..=MAX_ATTEMPTS {
            match self.update(id, title.clone(), content.clone()).await {
                Ok(updated) => return Ok(updated),
                Err(err) if is_busy_or_locked(&err) && attempt < MAX_ATTEMPTS => {
                    sleep(backoff).await;
                    backoff = backoff.saturating_mul(2);
                }
                Err(err) => return Err(err),
            }
        }

        unreachable!("loop always returns before reaching here");
    }
}

fn is_busy_or_locked(err: &crate::error::Error) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("database is locked") || msg.contains("busy")
}
