use std::path::Path;

use snafu::ResultExt;
use turso::{Builder, Connection};

use crate::db::post::PostRepo;
use crate::error::{DbBuilderSnafu, DbConnectSnafu};

use crate::Result;

pub async fn create_db_pool(filename: &Path) -> Result<Connection> {
    let db = Builder::new_local(filename.to_str().expect("DB path is required"))
        .build()
        .await
        .context(DbBuilderSnafu)?;
    let conn = db.connect().context(DbConnectSnafu)?;

    // Enable MVCC
    conn.pragma_update("journal_mode", "'mvcc'")
        .await
        .context(DbConnectSnafu)?;

    Ok(conn)
}

pub struct DbMapper {
    pub posts: PostRepo,
}

pub async fn create_db_mapper(filename: &Path) -> Result<DbMapper> {
    let pool = create_db_pool(filename).await?;
    Ok(DbMapper {
        posts: PostRepo::new(pool.clone()),
    })
}
