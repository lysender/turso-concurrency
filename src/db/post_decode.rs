use turso::Row;

use crate::Result;
use crate::db::turso_decode::{FromTursoRow, row_integer, row_text};
use crate::dto::PostDto;

impl FromTursoRow for PostDto {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row_integer(row, 0)?,
            title: row_text(row, 1)?,
            content: row_text(row, 2)?,
        })
    }
}
