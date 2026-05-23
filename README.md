# Turso Concurrency

A simple app to demonstrate tursodb's concurrent read/write behavior.

Observation:

Concurrent reads throws Misuse("concurrent use forbidden") error when using
a shared db connection and the connection is used to read the same row concurrently.

However, if you use multiple connections, concurrent reads work as expected without any errors.

Tested in versions:
- 0.5.3
- 0.6.1

## Initialize the Database

```
cd /path/to/app
tursodb db/sample.db < migrations/sample.sql
```

## ENV

```
DATABASE_DIR=/path/to/db_dir
```

## Run the App

```
cargo run
```
