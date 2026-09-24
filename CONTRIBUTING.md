# Contributing guidelines

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
