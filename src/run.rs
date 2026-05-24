use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::task::JoinHandle;
use tracing::info;

use crate::Result;
use crate::config::Config;
use crate::db::{DbMapper, PooledDbMapper, create_db_mapper, create_pooled_db_mapper};

const WORKERS: usize = 8;
const READS_PER_WORKER: usize = 10000;
const WRITES_PER_WORKER: usize = 1000;
const MAX_POST_ID: i64 = 5;

pub async fn run(config: Config) -> Result<()> {
    let db_file = config.db_dir.join("sample.db");

    if config.db_pooled {
        test_pooled(db_file, config.write).await
    } else {
        test_non_pooled(db_file, config.write).await
    }
}

async fn test_pooled(db_file: PathBuf, write: bool) -> Result<()> {
    if write {
        test_concurrent_writes_pooled(db_file.clone()).await?;
        info!("------------------------------------------------------------");
        info!("Starting pooled concurrent write with retry comparison");
        info!("------------------------------------------------------------");
        test_concurrent_writes_with_retry_pooled(db_file).await?;
    } else {
        test_concurrent_reads_pooled(db_file).await?;
    }

    Ok(())
}

async fn test_non_pooled(db_file: PathBuf, write: bool) -> Result<()> {
    if write {
        test_concurrent_writes(db_file.clone()).await?;
        info!("------------------------------------------------------------");
        info!("Starting non-shared concurrent write comparison");
        info!("------------------------------------------------------------");
        test_concurrent_writes_non_shared(db_file).await?;
    } else {
        test_concurrent_reads(db_file.clone()).await?;
        info!("------------------------------------------------------------");
        info!("Starting non-shared concurrent read comparison");
        info!("------------------------------------------------------------");
        test_concurrent_reads_non_shared(db_file).await?;
    }

    Ok(())
}

async fn test_concurrent_writes_pooled(db_file: PathBuf) -> Result<()> {
    let started_at = Instant::now();
    let mapper = create_pooled_db_mapper(db_file.as_path()).await?;
    let arc_mapper = Arc::new(mapper);

    let mut handles: Vec<JoinHandle<Result<WorkerResult>>> = Vec::with_capacity(WORKERS);

    for worker_id in 0..WORKERS {
        let worker_mapper = Arc::clone(&arc_mapper);
        handles.push(tokio::spawn(async move {
            run_write_worker_pooled(worker_mapper, worker_id).await
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
                        "Pooled worker finished"
                    );
                }
                Err(err) => {
                    dbg!(&err);
                }
            },
            Err(err) => {
                total_fail += WRITES_PER_WORKER;
                info!(
                    worker = worker_idx,
                    fail = WRITES_PER_WORKER,
                    error = %err,
                    "Pooled worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * WRITES_PER_WORKER;
    let duration_ms = started_at.elapsed().as_millis();
    info!(
        workers = WORKERS,
        writes_per_worker = WRITES_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        duration_ms,
        "Concurrent pooled write summary"
    );

    Ok(())
}

async fn test_concurrent_writes_with_retry_pooled(db_file: PathBuf) -> Result<()> {
    let started_at = Instant::now();
    let mapper = create_pooled_db_mapper(db_file.as_path()).await?;
    let arc_mapper = Arc::new(mapper);

    let mut handles: Vec<JoinHandle<Result<WorkerResult>>> = Vec::with_capacity(WORKERS);

    for worker_id in 0..WORKERS {
        let worker_mapper = Arc::clone(&arc_mapper);
        handles.push(tokio::spawn(async move {
            run_write_with_retry_worker_pooled(worker_mapper, worker_id).await
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
                        "Pooled worker finished"
                    );
                }
                Err(err) => {
                    dbg!(&err);
                }
            },
            Err(err) => {
                total_fail += WRITES_PER_WORKER;
                info!(
                    worker = worker_idx,
                    fail = WRITES_PER_WORKER,
                    error = %err,
                    "Pooled worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * WRITES_PER_WORKER;
    let duration_ms = started_at.elapsed().as_millis();
    info!(
        workers = WORKERS,
        writes_per_worker = WRITES_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        duration_ms,
        "Concurrent pooled write summary"
    );

    Ok(())
}

async fn test_concurrent_writes(db_file: PathBuf) -> Result<()> {
    let started_at = Instant::now();
    let mapper = create_db_mapper(db_file.as_path()).await?;
    let arc_mapper = Arc::new(mapper);

    let mut handles: Vec<JoinHandle<Result<WorkerResult>>> = Vec::with_capacity(WORKERS);

    for worker_id in 0..WORKERS {
        let worker_mapper = Arc::clone(&arc_mapper);
        handles.push(tokio::spawn(async move {
            run_write_worker(worker_mapper, worker_id).await
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
                        "Shared worker finished"
                    );
                }
                Err(err) => {
                    dbg!(&err);
                }
            },
            Err(err) => {
                total_fail += WRITES_PER_WORKER;
                info!(
                    worker = worker_idx,
                    fail = WRITES_PER_WORKER,
                    error = %err,
                    "Shared worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * WRITES_PER_WORKER;
    let duration_ms = started_at.elapsed().as_millis();
    info!(
        workers = WORKERS,
        writes_per_worker = WRITES_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        duration_ms,
        "Concurrent shared write summary"
    );

    Ok(())
}

async fn test_concurrent_writes_non_shared(db_file: PathBuf) -> Result<()> {
    let started_at = Instant::now();

    let mut handles: Vec<JoinHandle<Result<WorkerResult>>> = Vec::with_capacity(WORKERS);

    for worker_id in 0..WORKERS {
        let worker_db_file = db_file.clone();
        handles.push(tokio::spawn(async move {
            run_write_worker_non_shared(worker_db_file, worker_id).await
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
                        "Non-shared worker finished"
                    );
                }
                Err(err) => {
                    dbg!(&err);
                }
            },
            Err(err) => {
                total_fail += WRITES_PER_WORKER;
                info!(
                    worker = worker_idx,
                    fail = WRITES_PER_WORKER,
                    error = %err,
                    "Non-shared worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * WRITES_PER_WORKER;
    let duration_ms = started_at.elapsed().as_millis();
    info!(
        workers = WORKERS,
        writes_per_worker = WRITES_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        duration_ms,
        "Concurrent non-shared write summary"
    );

    Ok(())
}

async fn test_concurrent_reads_pooled(db_file: PathBuf) -> Result<()> {
    let started_at = Instant::now();
    let mapper = create_pooled_db_mapper(db_file.as_path()).await?;
    let arc_mapper = Arc::new(mapper);

    let mut handles: Vec<JoinHandle<Result<WorkerResult>>> = Vec::with_capacity(WORKERS);

    for worker_id in 0..WORKERS {
        let worker_mapper = Arc::clone(&arc_mapper);
        handles.push(tokio::spawn(async move {
            run_read_worker_pooled(worker_mapper, worker_id).await
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
                        "Pooled worker finished"
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
                    "Pooled worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * READS_PER_WORKER;
    let duration_ms = started_at.elapsed().as_millis();
    info!(
        workers = WORKERS,
        reads_per_worker = READS_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        duration_ms,
        "Concurrent pooled read summary"
    );

    Ok(())
}

async fn test_concurrent_reads(db_file: PathBuf) -> Result<()> {
    let started_at = Instant::now();
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
                        "Shared worker finished"
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
                    "Shared worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * READS_PER_WORKER;
    let duration_ms = started_at.elapsed().as_millis();
    info!(
        workers = WORKERS,
        reads_per_worker = READS_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        duration_ms,
        "Concurrent shared read summary"
    );

    Ok(())
}

async fn test_concurrent_reads_non_shared(db_file: PathBuf) -> Result<()> {
    let started_at = Instant::now();

    let mut handles: Vec<JoinHandle<Result<WorkerResult>>> = Vec::with_capacity(WORKERS);

    for worker_id in 0..WORKERS {
        let worker_db_file = db_file.clone();
        handles.push(tokio::spawn(async move {
            run_read_worker_non_shared(worker_db_file, worker_id).await
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
                        "Non-shared worker finished"
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
                    "Non-shared worker crashed"
                );
            }
        }
    }

    let attempts = WORKERS * READS_PER_WORKER;
    let duration_ms = started_at.elapsed().as_millis();
    info!(
        workers = WORKERS,
        reads_per_worker = READS_PER_WORKER,
        attempts,
        success = total_success,
        fail = total_fail,
        duration_ms,
        "Concurrent non-shared read summary"
    );

    Ok(())
}

struct WorkerResult {
    success: usize,
    fail: usize,
}

async fn run_read_worker(mapper: Arc<DbMapper>, worker_id: usize) -> Result<WorkerResult> {
    info!("Starting shared worker {worker_id}");

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

async fn run_write_worker(mapper: Arc<DbMapper>, worker_id: usize) -> Result<WorkerResult> {
    info!("Starting shared worker {worker_id}");

    let mut success: usize = 0;
    let mut fail: usize = 0;

    for _ in 0..WRITES_PER_WORKER {
        let id = rand::random_range(1..=MAX_POST_ID);
        let suffix = rand::random_range(1..=100);
        let title = format!("title - {suffix}");
        let content = format!("content - {suffix}");
        let res = mapper.posts.update(id, title, content).await;

        match res {
            Ok(true) => success += 1,
            Ok(false) => fail += 1,
            Err(e) => {
                dbg!(&e);
                fail += 1;
            }
        }
    }

    Ok(WorkerResult { success, fail })
}

async fn run_read_worker_non_shared(db_file: PathBuf, worker_id: usize) -> Result<WorkerResult> {
    info!("Starting non-shared worker {worker_id}");

    let mapper = create_db_mapper(db_file.as_path()).await?;

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

async fn run_write_worker_non_shared(db_file: PathBuf, worker_id: usize) -> Result<WorkerResult> {
    info!("Starting non-shared worker {worker_id}");

    let mapper = create_db_mapper(db_file.as_path()).await?;

    let mut success: usize = 0;
    let mut fail: usize = 0;

    for _ in 0..WRITES_PER_WORKER {
        let id = rand::random_range(1..=MAX_POST_ID);
        let suffix = rand::random_range(1..=100);
        let title = format!("title - {suffix}");
        let content = format!("content - {suffix}");
        let res = mapper.posts.update(id, title, content).await;

        match res {
            Ok(true) => success += 1,
            Ok(false) => fail += 1,
            Err(e) => {
                dbg!(&e);
                fail += 1;
            }
        }
    }

    Ok(WorkerResult { success, fail })
}

async fn run_read_worker_pooled(
    mapper: Arc<PooledDbMapper>,
    worker_id: usize,
) -> Result<WorkerResult> {
    info!("Starting pooled worker {worker_id}");

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

async fn run_write_worker_pooled(
    mapper: Arc<PooledDbMapper>,
    worker_id: usize,
) -> Result<WorkerResult> {
    info!("Starting pooled worker {worker_id}");

    let mut success: usize = 0;
    let mut fail: usize = 0;

    for _ in 0..WRITES_PER_WORKER {
        let id = rand::random_range(1..=MAX_POST_ID);
        let suffix = rand::random_range(1..=100);
        let title = format!("title - {suffix}");
        let content = format!("content - {suffix}");
        let res = mapper.posts.update(id, title, content).await;

        match res {
            Ok(true) => success += 1,
            Ok(false) => fail += 1,
            Err(e) => {
                dbg!(&e);
                fail += 1;
            }
        }
    }

    Ok(WorkerResult { success, fail })
}

async fn run_write_with_retry_worker_pooled(
    mapper: Arc<PooledDbMapper>,
    worker_id: usize,
) -> Result<WorkerResult> {
    info!("Starting pooled worker {worker_id}");

    let mut success: usize = 0;
    let mut fail: usize = 0;

    for _ in 0..WRITES_PER_WORKER {
        let id = rand::random_range(1..=MAX_POST_ID);
        let suffix = rand::random_range(1..=100);
        let title = format!("title - {suffix}");
        let content = format!("content - {suffix}");
        let res = mapper.posts.update_with_retry(id, title, content, 9).await;

        match res {
            Ok(true) => success += 1,
            Ok(false) => fail += 1,
            Err(e) => {
                dbg!(&e);
                fail += 1;
            }
        }
    }

    Ok(WorkerResult { success, fail })
}
