# Agent Notes

## Current Reality (verify before assuming more)
- This repo is a minimal Rust binary crate, not a workspace: `Cargo.toml` defines one package (`edition = "2024"`) and `src/main.rs` is the only code entrypoint.
- `src/main.rs` currently only prints `Hello, world!`; there is no Turso client logic yet.
- There is no CI config, task runner, formatter config, linter config, or pre-commit config checked in.

## Commands That Matter
- Run the app: `cargo run`
- Run tests (currently no test files, so this mainly verifies compile/test harness setup): `cargo test`

## Repo-Specific Files
- SQL lives under `migrations/`; `migrations/sample.sql` is a standalone sample migration and is not auto-applied by any script in this repo.
- `db/` is kept in git via `db/.gitignore`, while local Turso/SQLite artifacts in that directory (`*.db`, `*.db-wal`, `*.db-shm`) are ignored; do not commit generated DB files.

## Guardrails for Future Edits
- Keep guidance grounded in executable truth from this repo; README is currently minimal and does not describe runtime/setup beyond project intent.
- If adding concurrency/Turso behavior, document new run/setup steps in this file only after verifying commands actually work in-repo.
- Never run `git commit`, `git push`, `git commit --amend`, or any history-rewriting git command from an agent session; all commit/push/history actions are strictly user-only.
