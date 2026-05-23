use std::path::PathBuf;
use std::sync::Arc;

use tokio::task::JoinHandle;
use tracing::info;

use crate::Result;
use crate::config::Config;
use crate::db::{DbMapper, create_db_mapper};

const WORKERS: usize = 8;
const READS_PER_WORKER: usize = 100;
const MAX_POST_ID: i64 = 5;

pub async fn run(config: Config) -> Result<()> {
    let db_file = config.db_dir.join("sample.db");

    test_concurrent_reads(db_file).await
}

async fn test_concurrent_reads(db_file: PathBuf) -> Result<()> {
    let mapper = create_db_mapper(db_file.as_path()).await?;
    let arc_mapper = Arc::new(mapper);

    let mut handles: Vec<JoinHandle<Result<WorkerResult>>> = Vec::with_capacity(WORKERS);

    for worker_id in 0..WORKERS {
        let worker_mapper = Arc::clone(&arc_mapper);
        handles.push(tokio::spawn(async move {
            run_read_worker(worker_mapper, worker_id).await
        }));
    }

    let mut total_success = 0usize;
    let mut total_fail = 0usize;

    for (worker_idx, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(res) => match res {
                Ok(result) => {
                    total_success += result.success;
                    total_fail += result.fail;
                    info!(
                        worker = worker_idx,
                        success = result.success,
                        fail = result.fail,
                        "Worker finished"
                    );
                }
                Err(err) => {
                    dbg!(&err);
                }
            },
            Err(err) => {
                total_fail += READS_PER_WORKER;
                info!(
                    worker = worker_idx,
                    fail = READS_PER_WORKER,
                    error = %err,
                    "Worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * READS_PER_WORKER;
    info!(
        workers = WORKERS,
        reads_per_worker = READS_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        "Concurrent read summary"
    );

    Ok(())
}

struct WorkerResult {
    success: usize,
    fail: usize,
}

async fn run_read_worker(mapper: Arc<DbMapper>, worker_id: usize) -> Result<WorkerResult> {
    info!("Starting worker {worker_id}");

    let mut success: usize = 0;
    let mut fail: usize = 0;

    for _ in 0..READS_PER_WORKER {
        let id = rand::random_range(1..=MAX_POST_ID);
        let res = mapper.posts.get(id).await;

        match res {
            Ok(Some(_)) => success += 1,
            Ok(None) => fail += 1,
            Err(e) => {
                dbg!(&e);
                fail += 1;
            }
        }
    }

    Ok(WorkerResult { success, fail })
}
