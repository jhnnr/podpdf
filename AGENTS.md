# AGENTS.md

This file provides guidance to AI Agents when working with code in this repository.

## Cargo (Recommended)

This project uses [Rust](https://www.rust-lang.org/) and its built-in package manager and build system, [Cargo](https://doc.rust-lang.org/cargo/). All build, development, testing, and linting commands can be run from the repository root via `cargo <command>`.

### Quick Reference

- **`cargo build`** — Compile the project. Use `cargo build --release` for an optimized release build.
- **`cargo run`** — Build and run the web application.
- **`cargo test`** — Run all tests.
- **`cargo clippy`** — Run the linter to catch common mistakes and improve your code.
- **`cargo fmt`** — Auto-format all code to maintain a consistent style.
- **`cargo check`** — Check the project for errors without building the final executable. This is faster than a full build.
- **`cargo doc --open`** — Build and view the documentation for your project.

## Docker Development

This project is designed to be deployed as a containerized web application.

- **Build Docker image**: `docker build -t podpdf .`
- **Run Docker container**: `docker run -p 8080:8080 podpdf`
- **Docker Compose**: Example `docker-compose.yml` files can be used for more complex setups.
  - **Start stack**: `docker-compose up`
  - **Stop stack**: `docker-compose down`
  - **View logs**: `docker-compose logs -f`

## Comments

A comment must carry information the code cannot. If a reader could derive it from the code in front of them, delete it.

Comment the current state. Not what the code used to do, not what changed, not why it changed: git holds that. Where history explains the shape, state the reason instead, so "this used to reimplement the modal internals" becomes "thin wrapper over the shared Modal: duplicating its portal and focus trap is how dialogs drift apart". Future state goes in a TODO with an issue.

Write a comment when it does one of these four jobs:

- **Contract.** What a caller must know that the signature cannot say: preconditions, invariants, units, ownership and lifetime, thread-safety, error semantics, side effects. Document the contract of everything a caller outside the file can reach, and nothing else. Goes on the type/method/module as a doc comment (`///`).
- **Why.** The constraint the code satisfies, the bug it avoids, the alternative rejected and the reason.
- **Hazard.** "Must stay in sync with X", "order matters because Y", "do not remove, it prevents Z".
- **Map.** A short orientation at the top of a genuinely complex file: what it owns, and what it deliberately does not.

Never write:

- A comment that restates the next line. `// Handle drag start` above `handle_drag_start` is noise.
- Section banners or position markers: `// --- Types ---`, `// Helpers`, `// =====`.
- Step narration in a function body (`// Step 1:`, `// Then we`). If the steps need labels they need names: extract functions.
- Commented-out code. Delete it.
- Doc tags that restate the signature. `@param blob - The blob` says nothing; omit the tag.
- Docs on self-explanatory members with no constraint to state.

Two tests before keeping a comment:

- **Delete it.** Is any information lost? If not, it stays deleted.
- **Could a name carry it instead?** A better identifier, an extracted function, or a named constant beats a comment. Prefer the code change.

A comment at the end of a line usually decodes that line, and that is worth keeping: `{0x25, 0x50} // "%PDF"`, `50 * 1024 * 1024 // 50 MB`.

A reference is supplementary, never load-bearing: the comment must survive deleting it. `// See #1234` is a dead end; `// saving first loses every annotation (#6865)` is not. Prefer a spec (`RFC 3161`) or CVE where one applies.

A TODO needs an issue, not an owner: `// TODO(#1234): ...`. If it is not worth an issue, it is not worth a TODO. A question is not a TODO.

## Architecture Overview

This project is a web application written in Rust, providing a suite of tools for manipulating PDF files via a web interface and RESTful API endpoints.

### Project Goals

- **Performance & Safety**: Leverage Rust to build a fast, memory-safe, and concurrent application.
- **No Telemetry**: The application will not collect or transmit any user data. User privacy is paramount.
- **Web-Only**: The focus is exclusively on the web application, served via a backend server. There is no desktop application.
- **Stateless Backend**: The backend is designed to be stateless. Files are processed in memory or temporary storage and are not persisted on the server.
- **API First**: All functionality is exposed through a clear and consistent set of API endpoints.

### Key Components

- **Web Framework**: A modern Rust web framework (e.g., Axum, Actix-web) to handle HTTP requests and routing.
- **API Controllers**: API endpoints organized by function, following REST principles.
- **PDF Processing**: A core library in Rust for all PDF manipulation tasks.
- **Frontend**: A modern, lightweight Single-Page Application (SPA) that interacts with the backend API.
- **Configuration**: Configuration is managed through a configuration file (e.g., `config.toml`) and environment variables.

### Testing Strategy

- **Unit Tests**: Rust's built-in testing framework is used for unit testing individual functions and modules. Tests live alongside the code they are testing.
- **Integration Tests**: Located in the `tests/` directory, these tests validate the behavior of the application as a whole, including API endpoint functionality.
- **No fixed waits in tests**: Never sleep in tests. It causes flakes. Wait for the actual condition instead.

## Development Workflow

1.  **Local Development**: Run `cargo run` to start the local development server.
2.  **Quality Gate**: Run `cargo check`, `cargo fmt`, `cargo clippy`, and `cargo test` before submitting pull requests to ensure code quality and consistency.
3.  **Docker Testing**: Use `docker build .` and `docker run` to test the final containerized application.

## Communication Style

- Be direct and to the point.
- No apologies or conversational filler.
- Answer questions directly without preamble.
- Explain reasoning concisely when asked.
- Avoid unnecessary elaboration.

## Decision Making

- Ask clarifying questions before making assumptions.
- Stop and ask when uncertain about project-specific details.
- Confirm approach before making structural changes.
- Request guidance on preferences.
- Verify understanding of requirements before proceeding.
