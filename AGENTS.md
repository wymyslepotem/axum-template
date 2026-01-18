# AGENTS.md

## Project goals
- {{project_goal}}
- Keep the code clean, conventional, and easy to extend.
- Keep a strong production baseline: observability, request correlation, and consistent error responses.

## Working agreements
- All code comments and logs must be in English.
- Follow the existing project layout and naming conventions (e.g. `ops`, `dto`, `handlers`, `routes`).
- Avoid introducing new dependencies. Only add a crate when it is conventional for the Rust/Axum ecosystem and clearly justified by the use case (and prefer lightweight options).
- Keep handlers thin; push business logic into services as the project grows.
- Use `tracing` for logs/spans (avoid `println!`).
- Preserve `x-request-id` propagation and keep it consistent across the stack.
- Use the unified `AppError` model for errors returned from handlers.

## Change policy
- Prefer small, focused changes.
- Do not add heavy dependencies unless necessary; ask before adding new ones.
- After changes, run:
    - `cargo fmt`
    - `cargo clippy -- -D warnings`
    - `cargo test`
