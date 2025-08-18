# AGENTS.md - Development Guidelines for WICTK

## Build/Test/Lint Commands
- **Build**: `cargo build` or `cargo b` or `make build`
- **Test**: `cargo test` or `cargo t` or `make test`
- **Check**: `cargo check` or `make check`
- **Single test**: `cargo test test_name` (e.g., `cargo test parse_location`)
- **Workspace commands**: Run from root directory, targets all workspace members

## Project Structure
- **Workspace**: Multi-crate Rust project with `backend`, `client_logger`, `wictk_core`
- **Main service**: `backend/` contains Axum web server
- **Core logic**: `wictk_core/` contains shared business logic
- **Client**: `client_logger/` contains logging client

## Code Style Guidelines
- **Language**: Rust 2021 edition
- **Imports**: Use `use` statements, group by std/external/internal, alphabetical within groups
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/structs/enums
- **Error handling**: Use `anyhow::Error` for main functions, `Result<T, E>` for fallible operations
- **Async**: Use `tokio` runtime, `#[tokio::main]` for main, `async fn` for handlers
- **Logging**: Use `tracing` crate with `#[instrument]` macro for function tracing
- **Secrets**: Wrap sensitive data with `redact::Secret<T>`
- **JSON**: Use `serde` with `#[derive(Serialize, Deserialize)]`
- **Testing**: Use `#[cfg(test)]` modules, `pretty_assertions` for test assertions