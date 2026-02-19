# AGENTS.md

Guidance for coding agents working in this repository.

## Project
- Name: `sprint-tasks` (Rust CLI)
- Purpose: List and create tasks in the active Jira sprint.

## Common Commands
- Build: `cargo build`
- Run: `cargo run -- <args>`
- Test: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`

## Repo Structure
- `src/` — application source code
- `Cargo.toml` — dependencies and package metadata
- `README.md` — usage and setup docs
- `justfile` — helper tasks for install/dev workflows

## Change Guidelines
- Keep changes focused and minimal.
- Prefer idiomatic Rust and clear error messages.
- Update `README.md` when user-facing CLI behavior changes.
- Run format/lint/tests when making functional changes.

## Safety / Scope
- Do not commit secrets or local config values.
- Do not modify CI/release behavior unless explicitly requested.
- Avoid broad refactors unless requested.
