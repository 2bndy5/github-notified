# Agent and Contributor Guide

This document describes the repository conventions for coding agents and contributors working on GitHub Notified.

## Repository Shape

- Rust application code lives under `src/`.
- Browser API adapters live under `src/browser/`.
- GitHub API code and data models live under `src/github/`.
- Notification workflow and filtering live under `src/notifications/`.
- Extension packaging and generated HTML/JavaScript live in `scripts/package.nu`.
- Static icons live under `static/`.
- `dist/` and `target/` are generated outputs and should not be edited by hand.

Read `Architecture.md` before changing event wiring, polling, storage, or packaging behavior.

## Development Prerequisites

Install the stable Rust toolchain, the WebAssembly target, Nushell, `nur`, and the matching `wasm-bindgen-cli` when building the extension:

```sh
rustup target add wasm32-unknown-unknown
cargo install nur
cargo install wasm-bindgen-cli
```

`cargo binstall` can be used alternatively (instead of `cargo install`) if `cargo-binstall` is installed.

The CLI version should match the `wasm-bindgen` crate version used by the project.

## Standard Checks

Use the repository tasks when possible:

```sh
nur test    # Native unit tests
nur lint    # Formatting check and Clippy with warnings denied
nur fmt     # Format Rust code
nur check   # Fast native compile check
nur build --release
nur package --release
nur clean
```

A source-only change normally needs `nur test` and `nur lint`. A change affecting WebAssembly bindings, generated pages, manifests, or browser behavior should also be checked with `nur build --release` or the narrowest applicable packaging target.

## Implementation Guidelines

- Keep business logic in Rust and keep raw browser API declarations in `src/js_bridge.rs` or the appropriate `src/browser` adapter.
- Prefer pure functions for filtering, URL conversion, and configuration behavior so they can be covered by native tests.
- Preserve `#[cfg(target_arch = "wasm32")]` boundaries. Native tests must compile without browser globals.
- Use existing `Config`, storage keys, model types, and browser adapters instead of duplicating serialized structures or API calls.
- Keep GitHub API and web root URLs configurable; do not hard-code `github.com` into Enterprise-sensitive behavior.
- Do not add API mutations for marking notifications read, replying, or merging. Those actions belong to GitHub's web interface.
- Never log or commit personal access tokens. Avoid including token values in errors, tests, examples, or documentation.
- Keep public APIs and serialized field names stable unless a change explicitly includes migration or compatibility handling.
- Keep changes focused. Do not edit generated `dist/` files directly and do not include `target/` artifacts in changes.

## Testing Expectations

Add or update focused tests beside pure Rust behavior. Useful coverage areas include:

- Configuration defaults and GitHub Enterprise URL construction.
- Notification whitelist and blacklist behavior.
- New-notification detection against the cached IDs.
- GitHub API header and query URL construction.
- Conversion of API URLs to issue, pull request, commit, discussion, and comment links.

Browser-only behavior should be validated through the smallest available packaged extension build and manual loading in Chromium or Firefox when relevant. Keep browser API callbacks and generated-page changes easy to exercise without requiring a live GitHub token.

## Review Checklist

Before submitting a change, confirm:

- `nur test` passes.
- `nur lint` passes.
- New behavior has a focused test when it is testable without a browser.
- WASM-only imports and browser calls are correctly gated.
- Error paths update the user-visible badge/title consistently where applicable.
- Settings changes still take effect without requiring an extension reload.
- Documentation and generated packaging inputs agree with the current code.
- No secrets, build artifacts, or unrelated formatting changes are included.
