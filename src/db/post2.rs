use std::cmp::min;
use std::time::Duration;

use snafu::ResultExt;
use tokio::time::sleep;

use crate::db::db_pool::DbPool;
use crate::db::turso_decode::collect_row;
use crate::db::turso_params::{integer_param, new_query_params, text_param};
use crate::dto::PostDto;
use crate::error::{DbPrepareSnafu, DbStatementSnafu};
use crate::{Error, Result};

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

    pub async fn update_with_retry(
        &self,
        id: i64,
        title: String,
        content: String,
        max_retries: usize,
    ) -> Result<bool> {
        let mut attempts = 0;
        let mut delay = Duration::from_millis(100);
        let max_delay = Duration::from_secs(2);

        loop {
            match self.update(id, title.clone(), content.clone()).await {
                Ok(result) => return Ok(result),
                Err(Error::DbStatement { source }) => match source {
                    turso::Error::Busy(..) => {
                        attempts += 1;
                        if attempts >= max_retries {
                            return Err(Error::DbStatement { source });
                        }

                        sleep(delay).await;
                        delay = min(delay.saturating_mul(2), max_delay);
                        // Retries...
                    }
                    _ => {
                        return Err(Error::DbStatement { source });
                    }
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
