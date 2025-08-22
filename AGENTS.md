# AGENTS.md

## Build & Test Commands
- Build: `cargo build`
- Run: `cargo run`
- Test all: `cargo test`
- Test single: `cargo test can_load_rectangle_rendering_shader`
- Check: `cargo check`
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Code Style Guidelines

### Imports
- Group imports by source (std, external crates, internal modules)
- Order: std first, then external crates alphabetically, then internal modules

### Types & Structs
- Use `#[derive(Debug)]` for structs
- Public fields use `pub` keyword
- Implement constructor methods as `new()`

### Error Handling
- Use `anyhow::Result` for functions that can fail
- Return `Err(anyhow!("error message"))` for errors
- Use `?` operator for propagating errors

### Naming
- Use snake_case for functions, variables, and modules
- Use PascalCase for types, traits, and enums
- Descriptive, clear names preferred over abbreviations

### Tests
- Write unit tests in the same file in a `mod tests` submodule
- Use `#[test]` attribute for test functions
- Name tests to describe what they're testing