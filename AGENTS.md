# Agent Notes

## Repo Shape
- Single Rust binary crate (not a workspace): `Cargo.toml` defines one package (`edition = "2024"`).
- Entrypoint flow is `src/main.rs` -> `run_command()` -> `src/run.rs::run()`.
- Turso integration lives in `src/db/`; app currently just opens `${DATABASE_DIR}/sample.db` and exits.

## Commands
- Run app (required env): `DATABASE_DIR=db cargo run`
- Smoke-check compile/test harness: `cargo test` (currently runs 0 tests)

## Runtime Requirements
- `DATABASE_DIR` is mandatory (`Config::build()` errors immediately if missing/empty).
- `RUST_LOG_MAX` is optional but must parse as a `tracing::Level` (invalid value exits at startup).

## Database and Migrations
- `migrations/sample.sql` is not auto-applied; apply it manually (README uses `tursodb db/sample.db < migrations/sample.sql`).
- `db/` is intentionally tracked via `db/.gitignore`.
- Only `*.db`, `*.db-wal`, and `*.db-shm` are ignored in `db/`; local runs may create other sidecars (for example `*.db-log`) that show up as untracked.

## Tooling Reality
- No CI workflows, task runner, linter config, formatter config, or pre-commit config are checked in.

## Git Guardrail
- Agents must not run `git commit`, `git commit --amend`, `git push`, or any history-rewriting git command in this repo.
- Leave all changes unstaged or staged only as requested; the human user is responsible for creating commits.
