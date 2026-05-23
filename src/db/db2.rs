use std::path::Path;

use crate::Result;
use crate::db::db_pool::DbPool;
use crate::db::post2::PooledPostRepo;

const DB2_POOL_SIZE: usize = 8;

pub struct PooledDbMapper {
    pub posts: PooledPostRepo,
}

pub async fn create_pooled_db_mapper(filename: &Path) -> Result<PooledDbMapper> {
    let pool = DbPool::new(filename, DB2_POOL_SIZE).await?;
    Ok(PooledDbMapper {
        posts: PooledPostRepo::new(pool),
    })
}
