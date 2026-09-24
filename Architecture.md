# Architecture

GitHub Notified is a browser extension whose application logic is written in Rust and compiled to WebAssembly. A small amount of generated JavaScript loads the WebAssembly module and provides the browser extension entrypoint required by Manifest V3.

## Runtime Overview

The extension has one Rust library crate with six public modules:

- `config`: Settings, defaults, URL construction, and persistence-facing configuration behavior.
- `github`: GitHub REST API access, response models, and conversion from API URLs to browser URLs.
- `notifications`: Notification filtering, new-item detection, and the polling workflow.
- `browser`: Browser-facing operations such as alarms, action badge/title updates, tabs, and desktop notifications.
- `storage`: Typed helpers for `chrome.storage.local`.
- `js_bridge`: Low-level `wasm-bindgen` declarations for Chrome Extension APIs.

`src/lib.rs` is the application entrypoint. It exports the WebAssembly functions used by the generated background script and wires browser events to the notification service.

## Startup and Event Flow

The packaging script generates `background.js`. Its startup sequence is:

1. Load the WebAssembly module.
2. Register `__register_on_installed()` synchronously. This preserves the browser's requirement that the `runtime.onInstalled` listener be registered during startup.
3. Await `__bg_start()`.

`__bg_start()` registers the remaining event handlers and performs the first poll:

- The alarm listener runs `notifications::service::poll_and_update` at the configured interval.
- A toolbar action click opens or focuses the GitHub notifications page.
- A local-storage settings change reschedules polling and immediately polls with the new configuration.
- A desktop notification click opens or focuses the notification's web URL.
- Startup loads configuration, schedules the alarm, and polls once.

Callbacks registered with the browser APIs are long-lived WebAssembly closures. Asynchronous work is started with `wasm_bindgen_futures::spawn_local`.

## Notification Polling

`notifications::service::poll_and_update` is the main application workflow:

1. Validate that a token is configured. If it is missing, show an error badge and stop.
2. Create a `GitHubClient` from the configured token and API URL.
3. Load the previous notification summary cache from local storage.
4. Fetch up to 100 notifications from GitHub's `/notifications` endpoint, optionally using `participating=true`.
5. Apply the configured repository whitelist or blacklist.
6. Update the toolbar badge and title with the filtered unread count.
7. Compare filtered notification IDs with the previous cache and show desktop notifications for newly observed items.
8. Convert each item to a browser-openable URL and save a compact notification summary cache.

The service records the last API error in local storage and uses the error badge/title for authentication, rate-limit, network, and response parsing failures. On non-WASM targets, the polling routine is a no-op so the native test suite can exercise pure Rust logic without browser APIs.

## Module Boundaries

### GitHub API

`github::client::GitHubClient` owns API URL construction, authentication headers, and WASM HTTP requests. It uses a classic personal access token with the `notifications` scope and sends GitHub's current REST API version header.

`github::models` contains the serialized notification data types. `github::urls` translates API resource URLs into links for GitHub.com or a configured GitHub Enterprise web root, including issue and pull-request comment anchors.

### Configuration and Storage

`config::Config` is the shared settings contract between the generated options page and Rust. The options form mirrors its serialized fields. On WASM, `Config::load` and `Config::save` use `storage::get` and `storage::set`; native builds use defaults and no-op persistence for tests.

`storage` is deliberately typed at its public boundary and handles JSON-compatible serialization through `serde-wasm-bindgen`. `js_bridge` contains only raw browser API declarations, keeping API-specific JavaScript bindings out of the domain modules.

### Browser Integration

The `browser` module provides small, behavior-oriented adapters. Domain code calls operations such as setting the badge, scheduling an alarm, opening a tab, or showing a desktop notification without constructing browser API objects itself. Each adapter has non-WASM fallbacks where needed so native tests compile cleanly.

## Build and Packaging

The repository uses Nushell and `nur` for repeatable workflows. `nur build` and `nur package` call `scripts/package.nu`, which:

1. Builds the Rust library for `wasm32-unknown-unknown`.
2. Runs `wasm-bindgen --target web` to create JavaScript glue and type declarations.
3. Copies static icons.
4. Generates a Manifest V3 manifest for Chromium or Firefox.
5. Generates `background.js`, `options.html`, and `options.js`.
6. Optionally creates a ZIP archive for distribution.

The generated extension directories are `dist/chromium/` and `dist/firefox/`. They are packaging outputs, not source modules. Changes to the options page or background shim should be made in `scripts/package.nu` and regenerated rather than edited in `dist`.

## Design Constraints

- Keep mutable GitHub actions in the web interface. The extension reads notifications and does not mark, reply to, or merge items through the API.
- Preserve support for both GitHub.com and GitHub Enterprise by keeping API and web root URLs separate.
- Keep browser API calls behind `browser`, `storage`, and `js_bridge` so pure logic remains testable on native targets.
- Treat `Config` serialization as a compatibility boundary with the generated options page.
- Register required browser listeners synchronously where the browser API requires it, then perform asynchronous work after registration.
