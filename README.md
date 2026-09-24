# GitHub Notified ![gh-logo](static/icon-128.png)

A simple browser extension for GitHub notifications written in Rust.

![Manifest V3][mv3-badge]
![Rust Wasm][rs-wasm-badge]
[![MIT License][license-badge]](LICENSE)

Ported from [sindresorhus/notifier-for-github](https://github.com/sindresorhus/notifier-for-github) to Rust.

## Highlights

- 🦀 **Uses Rust & WebAssembly**: The only JavaScript code is used to load the WASM binary (and in the options page).
- 🔑 **GitHub Classic Personal Access Token (PAT)**: Authenticates securely via `Authorization: Bearer <token>` using modern GitHub REST API headers (`X-GitHub-Api-Version: 2022-11-28`).
- 🔒 **Read-Only Scope**: Only requires the `notifications` scope. All mutable actions (marking read, replying, merging) safely forward to GitHub's web interface.
- 🔔 **Desktop Alerts & Toolbar Badge**: Displays live unread notification count badge and triggers system desktop notifications on new events.
- 🏢 **GitHub Enterprise Support**: Supports custom API endpoints and web root URLs.
- 🎯 **Repository & Participation Filtering**: Option to filter only participating items, or configure whitelist/blacklist repository filters.
- 🌐 **Cross-Browser**: Compatible with Chromium (Chrome, Edge, Brave, Opera) and Firefox.

## Token Setup (Classic PAT)

To use GitHub Notified, create a classic personal access token on GitHub:

1. Go to **[GitHub Token Settings (classic)](https://github.com/settings/tokens)** (Settings → Developer Settings → Personal access tokens → Tokens (classic)).
2. Click **Generate new token (classic)**.
3. Give the token a name and set an expiration.
4. Under **Select scopes**, check **`notifications`**.
5. Click **Generate token** and copy your `ghp_...` token.
6. Open the extension's **Options** page and paste your token into the field. Click **Test Connection** to verify.

> [!note]
> The GitHub REST API `/notifications` endpoint does **not** support fine-grained PATs. You must use a classic PAT (`ghp_...`).


[license-badge]: https://img.shields.io/badge/License-MIT-green
[rs-wasm-badge]: https://img.shields.io/badge/Rust-Wasm-orange
[mv3-badge]: https://img.shields.io/badge/Manifest-V3-4285F4
