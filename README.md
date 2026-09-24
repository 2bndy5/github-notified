# GitHub Notified

<p align="center">
  <img src="static/icon-128.png" width="96" alt="GitHub Notified Logo" />
</p>

<p align="center">
  <b>A modern browser extension for GitHub notifications written in Rust and WebAssembly with <a href="https://github.com/0xsouravm/oxichrome">Oxichrome</a>.</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Manifest-V3-4285F4.svg" alt="Manifest V3" />
  <img src="https://img.shields.io/badge/Rust-Wasm-orange.svg" alt="Rust Wasm" />
  <img src="https://img.shields.io/badge/UI-Leptos_0.8-blue.svg" alt="Leptos" />
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT License" />
</p>

Ported from [sindresorhus/notifier-for-github](https://github.com/sindresorhus/notifier-for-github) to Rust.

---

## Highlights

- 🦀 **100% Rust & WebAssembly**: Zero handwritten JavaScript; built on [Oxichrome](https://github.com/0xsouravm/oxichrome).
- 🔑 **GitHub Classic Personal Access Token (PAT)**: Authenticates securely via `Authorization: Bearer <token>` using modern GitHub REST API headers (`X-GitHub-Api-Version: 2022-11-28`).
- 🔒 **Read-Only Scope**: Only requires the `notifications` scope. All mutable actions (marking read, replying, merging) safely forward to GitHub's web interface.
- ⚡ **Reactive UI with Leptos**: Fast popup for unread notifications and settings page for token configuration.
- 🔔 **Desktop Alerts & Toolbar Badge**: Displays live unread notification count badge and triggers system desktop notifications on new events.
- 🏢 **GitHub Enterprise Support**: Supports custom API endpoints and web root URLs.
- 🎯 **Repository & Participation Filtering**: Option to filter only participating items, or configure whitelist/blacklist repository filters.
- 🌐 **Cross-Browser**: Compatible with Chromium (Chrome, Edge, Brave, Opera) and Firefox.
- 🚀 **Automated CI/CD**: Complete GitHub Actions workflow for formatting (`cargo fmt`), linting (`cargo clippy`), unit tests (`cargo test`), and multi-browser release artifact packaging.

---

## Token Setup (Classic PAT)

To use GitHub Notified, create a classic personal access token on GitHub:

1. Go to **[GitHub Token Settings (classic)](https://github.com/settings/tokens)** (Settings → Developer Settings → Personal access tokens → Tokens (classic)).
2. Click **Generate new token (classic)**.
3. Give the token a name and set an expiration.
4. Under **Select scopes**, check **`notifications`**.
5. Click **Generate token** and copy your `ghp_...` token.
6. Open the extension's **Options** page and paste your token into the field. Click **Test Connection** to verify.

> **Note:** The GitHub REST API `/notifications` endpoint does **not** support fine-grained PATs. You must use a classic PAT (`ghp_...`).

---

## Project Structure

```text
github-notified/
├── .github/workflows/
│   ├── ci.yml                 # Linting, testing, and multi-browser builds
│   └── release.yml            # Automatic release tagging and asset uploads
├── src/
│   ├── lib.rs                 # Extension entrypoint (oxichrome macros)
│   ├── config.rs              # Configuration & storage models
│   ├── github/
│   │   ├── client.rs          # GitHub REST API client (Bearer auth, classic PAT)
│   │   ├── models.rs          # API response structs
│   │   └── urls.rs            # Web URL translation for issues, PRs, and comments
│   ├── browser/
│   │   ├── action.rs          # Badge text & color manipulation
│   │   ├── alarms.rs          # Periodic background check alarms
│   │   ├── notifications.rs   # Desktop notifications
│   │   └── tabs.rs            # Tab focus & reuse
│   ├── notifications/
│   │   ├── service.rs         # Polling engine, diffing, and badge updates
│   │   └── filter.rs          # Whitelist/blacklist repository filter
│   └── ui/
│       ├── popup.rs           # Leptos popup component
│       └── options.rs         # Leptos options page component
├── static/                    # Icons and extension assets
├── scripts/
│   └── package.nu             # Cross-platform Nushell packaging script
├── nurfile                    # Tasks for nur taskrunner
└── Cargo.toml
```

---

## Local Development & Building (with nur taskrunner)

This project uses **[nur](https://github.com/nur-taskrunner/nur)** (a task runner powered by [Nushell](https://www.nushell.sh/)) for cross-platform developer workflows.

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (stable) with wasm target:
  ```sh
  rustup target add wasm32-unknown-unknown
  ```
- [Nushell](https://www.nushell.sh/) & [nur](https://github.com/nur-taskrunner/nur):
  ```sh
  cargo install nur
  ```

### Available Tasks with `nur`
```sh
# List all available tasks
nur --list

# Run native unit tests
nur test

# Check code formatting & Clippy linter
nur lint

# Format code
nur fmt

# Build extension directories (Chromium & Firefox)
nur build --release

# Package extensions into distribution zips
nur package --release

# Clean build artifacts
nur clean
```

*(You can also invoke Nushell scripts directly: `nu scripts/package.nu --release`)*

The packaged outputs will be created in:
- `dist/chromium/` & `dist/github-notified-chromium.zip`
- `dist/firefox/` & `dist/github-notified-firefox.zip`

---

## Loading the Extension in Your Browser

### Chromium (Google Chrome, Edge, Brave, Opera)
1. Open `chrome://extensions/`
2. Enable **Developer mode** (top right toggle)
3. Click **Load unpacked**
4. Select the `dist/chromium` folder

### Firefox
1. Open `about:debugging#/runtime/this-firefox`
2. Click **Load Temporary Add-on...**
3. Select `dist/firefox/manifest.json`

---

## License

MIT © [GitHub Notified Contributors](LICENSE)
