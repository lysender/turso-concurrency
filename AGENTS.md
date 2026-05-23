# Agent Notes

## Repo Shape
- Single Rust binary crate (not a workspace): `Cargo.toml` defines one package (`edition = "2024"`).
- Entrypoint flow is `src/main.rs` -> `run_command()` -> `src/run.rs::run()`.
- Turso integration lives in `src/db/`; main behavior is a concurrency benchmark harness in `src/run.rs` (shared connection vs pool, reads vs writes).
- DB mapper split is intentional: `src/db/db.rs` + `src/db/post.rs` for a shared single connection, `src/db/db2.rs` + `src/db/post2.rs` for pooled connections (`src/db/db_pool.rs`).

## Commands
- Create DB first: `tursodb db/sample.db < migrations/sample.sql`
- Run default mode (shared connection, concurrent reads): `DATABASE_DIR=db cargo run`
- Run pooled mode: `DATABASE_DIR=db cargo run -- --pooled`
- Run write benchmark instead of read benchmark: add `--write` (can combine with `--pooled`).
- Verify quickly: `cargo test` (currently exercises `src/db/db_pool.rs` tests).

## Runtime Requirements
- `DATABASE_DIR` is mandatory (`Config::build()` errors immediately if missing/empty).
- `RUST_LOG_MAX` is optional but must parse as a `tracing::Level` (invalid value exits at startup).
- CLI flags are parsed via clap in `src/config.rs`: only `--pooled` and `--write` are supported.

## Database and Migrations
- `migrations/sample.sql` is not auto-applied; apply it manually (README uses `tursodb db/sample.db < migrations/sample.sql`).
- `db/` is intentionally tracked via `db/.gitignore`.
- `db/.gitignore` ignores `*.db*`, so `sample.db` and sidecars (`-wal`, `-shm`, `-log`, etc.) stay untracked.
- Migration seeds IDs `1..10`, but benchmark reads/writes only sample IDs `1..=5` (`MAX_POST_ID` in `src/run.rs`).

## Tooling Reality
- No CI workflows, task runner, linter config, formatter config, or pre-commit config are checked in.
- Repo-local OpenCode config exists at `.opencode/opencode.json` and enables local MCP server command `tursodb --mcp`.

## Git Guardrail
- Agents must not run `git commit`, `git commit --amend`, `git push`, or any history-rewriting git command in this repo.
- Leave all changes unstaged or staged only as requested; the human user is responsible for creating commits.
