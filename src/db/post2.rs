use snafu::ResultExt;

use crate::Result;
use crate::db::db_pool::DbPool;
use crate::db::turso_decode::collect_row;
use crate::db::turso_params::{integer_param, new_query_params};
use crate::dto::PostDto;
use crate::error::DbPrepareSnafu;

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
}
