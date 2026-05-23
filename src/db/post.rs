use snafu::ResultExt;
use turso::{Connection, Row};

use crate::Result;
use crate::db::turso_decode::{FromTursoRow, collect_row, row_integer, row_text};
use crate::db::turso_params::{integer_param, new_query_params};
use crate::dto::PostDto;
use crate::error::DbPrepareSnafu;

impl FromTursoRow for PostDto {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row_integer(row, 0)?,
            title: row_text(row, 1)?,
            content: row_text(row, 2)?,
        })
    }
}

pub struct PostRepo {
    db_pool: Connection,
}

impl PostRepo {
    pub fn new(db_pool: Connection) -> Self {
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

        let mut stmt = self.db_pool.prepare(query).await.context(DbPrepareSnafu)?;
        let row_result = stmt.query_row(q_params).await;
        let dto: Option<PostDto> = collect_row(row_result)?;
        Ok(dto)
    }
}
