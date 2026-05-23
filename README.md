# Turso Concurrency

A simple app to demonstrate tursodb's concurrent read/write behavior.

Observation:

Concurrent reads throws Misuse("concurrent use forbidden") error when using
a shared db connection and the connection is used to read the same row concurrently.

However, if you use multiple connections, concurrent reads work as expected without any errors.

Tested in versions:
- 0.5.3
- 0.6.1

Therefore, I'm creating a DB Pool so that concurrent reads of the same connection does not
end up reading the same row. Each db operation will need to acquire a connection from the pool,
then release it once the operation is done.

Below are the run modes to demonstrate the difference.

## Initialize the Database

```
cd /path/to/app
tursodb db/sample.db < migrations/sample.sql
```

## ENV

```
DATABASE_DIR=/path/to/db_dir
```

## Run the App (Two Modes)

Compare both modes to see the difference between a shared single connection and a connection pool under concurrent reads.

Default mode - shared single connection across multiple workers:
(This will throw the Misuse error when concurrent reads happen on the same row)

```
cargo run
```

Pooled mode (uses `--pooled`):
(This will work as expected but slightly slower due to the overhead of acquiring and releasing connections from the pool)

```
cargo run -- --pooled
```
