# scripts/package.nu
# Cross-platform Nushell packaging script for github-notified browser extension

def make_zip [source_dir: string, output_zip: string] {
    let abs_source = ($source_dir | path expand)
    let abs_zip = ($output_zip | path expand)

    if ($abs_zip | path exists) {
        rm -f $abs_zip
    }

    if (which tar | is-not-empty) {
        ^tar -a -cf $abs_zip -C $abs_source .
    } else if (which zip | is-not-empty) {
        cd $abs_source
        ^zip -r $abs_zip .
    } else {
        error make { msg: "Neither tar nor zip found on system to create archive" }
    }
}

export def package_extension [
    --target: string = "all", # chromium, firefox, or all
    --release                 # whether to build with --release
] {
    let profile = if $release { "release" } else { "debug" }
    print $"==> Packaging github-notified extension \(($profile)\)..."

    # Read package metadata from Cargo.toml (nushell parses TOML natively)
    let cargo_toml = (open Cargo.toml)
    let pkg_version = $cargo_toml.package.version
    let pkg_description = $cargo_toml.package.description

    # 1. Compile to wasm32-unknown-unknown
    let cargo_args = if $release {
        ["build", "--lib", "--release", "--target", "wasm32-unknown-unknown"]
    } else {
        ["build", "--lib", "--target", "wasm32-unknown-unknown"]
    }

    print $"==> Running cargo ($cargo_args | str join ' ')..."
    cargo ...$cargo_args

    let wasm_file = (["target", "wasm32-unknown-unknown", $profile, "github_notified.wasm"] | path join)
    if not ($wasm_file | path exists) {
        error make { msg: $"Wasm file not found at ($wasm_file)" }
    }

    # wasm-bindgen is required to produce the JS glue that background.js/popup.js/options.js import
    if (which wasm-bindgen | is-empty) {
        error make { msg: "wasm-bindgen CLI not found on PATH. Install it with `cargo install wasm-bindgen-cli` (matching the `wasm-bindgen` crate version) before packaging." }
    }

    let dist_dir = "dist"
    let targets = if $target == "all" { ["chromium", "firefox"] } else { [$target] }

    for t in $targets {
        let out_dir = ([$dist_dir, $t] | path join)
        let wasm_out = ([$out_dir, "wasm"] | path join)

        if ($out_dir | path exists) {
            rm -rf $out_dir
        }
        mkdir $wasm_out

        print $"==> Processing ($t) extension distribution at ($out_dir)..."

        wasm-bindgen --target web --out-dir $wasm_out --out-name "github_notified" $wasm_file

        # Copy static icon assets
        if ("static" | path exists) {
            for asset in (ls static/icon-*.png) {
                cp $asset.name $out_dir
            }
        }

        # Generate Manifest V3
        let base_manifest = {
            manifest_version: 3,
            name: "GitHub Notified",
            version: $pkg_version,
            description: $pkg_description,
            permissions: ["storage", "alarms", "notifications", "tabs"],
            host_permissions: ["https://api.github.com/*", "https://github.com/*"],
            icons: {
                "128": "icon-128.png"
            },
            action: {
                default_icon: {
                    "128": "icon-128.png"
                }
            },
            options_ui: {
                page: "options.html",
                open_in_tab: true
            },
            content_security_policy: {
                extension_pages: "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
            },
            web_accessible_resources: [
                {
                    resources: ["wasm/*", "icon-*.png"],
                    matches: ["<all_urls>"]
                }
            ]
        }

        let manifest = if $t == "chromium" {
            $base_manifest | insert background {
                service_worker: "background.js",
                type: "module"
            }
        } else {
            $base_manifest
                | insert background {
                    scripts: ["background.js"],
                    type: "module"
                }
                | insert browser_specific_settings {
                    gecko: {
                        id: "github-notified@github-notified.dev"
                    }
                }
        }

        $manifest | to json -i 2 | save -f ([$out_dir, "manifest.json"] | path join)

        # background.js shim - registers the onInstalled listener synchronously (MV3
        # requirement) then starts the polling/alarm/notification-click handlers.
        let bg_js = r#####'// Auto-generated shim for github-notified
import init, { __bg_start, __register_on_installed } from './wasm/github_notified.js';

async function start() {
    try {
        await init();
        __register_on_installed();
        await __bg_start();
        console.log('[github-notified] Background service worker initialized.');
    } catch (e) {
        console.error('[github-notified] Failed to initialize WASM:', e);
    }
}

start();
'#####
        $bg_js | save -f ([$out_dir, "background.js"] | path join)

        # options.html & options.js - form fields mirror the Rust `Config` struct exactly
        # (token, api_url, root_url, poll interval, notification toggles, repo filter mode/list)
        let options_html = r#####'<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>GitHub Notified - Settings</title>
    <style>
        html, body { margin: 0; padding: 0; background: #f6f8fa; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color: #1f2328; }
        form { max-width: 480px; margin: 24px auto; padding: 24px; background: #fff; border: 1px solid #d0d7de; border-radius: 6px; }
        h1 { font-size: 16px; margin: 0 0 16px; }
        label { display: block; font-weight: 600; margin: 16px 0 4px; font-size: 13px; }
        .hint { font-weight: normal; color: #57606a; font-size: 12px; margin-top: 2px; }
        input[type="text"], input[type="password"], input[type="number"], select, textarea {
            width: 100%; box-sizing: border-box; padding: 6px 8px; border: 1px solid #d0d7de; border-radius: 6px; font-size: 13px;
        }
        textarea { min-height: 80px; font-family: monospace; }
        .checkbox-row { display: flex; align-items: center; gap: 8px; margin: 12px 0 0; }
        .checkbox-row label { margin: 0; font-weight: normal; }
        .actions { display: flex; align-items: center; gap: 12px; margin-top: 24px; }
        button { padding: 6px 14px; border-radius: 6px; border: 1px solid #d0d7de; background: #f6f8fa; cursor: pointer; font-size: 13px; }
        button.primary { background: #1f883d; border-color: #1f883d; color: #fff; }
        #status { font-size: 12px; color: #57606a; }
    </style>
</head>
<body>
    <form id="settings-form">
        <h1>GitHub Notified - Settings</h1>

        <label for="token">Personal Access Token (classic)
            <div class="hint">Requires the "notifications" scope. Fine-grained tokens are not supported by the GitHub notifications API.</div>
        </label>
        <input type="password" id="token" autocomplete="off">

        <label for="api_url">API URL</label>
        <input type="text" id="api_url">

        <label for="root_url">Web Root URL</label>
        <input type="text" id="root_url">

        <label for="poll_interval_mins">Check interval (minutes)</label>
        <input type="number" id="poll_interval_mins" min="1" step="1">

        <label for="repo_filter_mode">Repository filter</label>
        <select id="repo_filter_mode">
            <option value="None">No filter</option>
            <option value="Whitelist">Only these repositories</option>
            <option value="Blacklist">Exclude these repositories</option>
        </select>

        <label for="repo_filter_list">Repositories
            <div class="hint">One "owner/repo" per line. Used when a filter above is selected.</div>
        </label>
        <textarea id="repo_filter_list" placeholder="owner/repo"></textarea>

        <div class="checkbox-row">
            <input type="checkbox" id="only_participating">
            <label for="only_participating">Only show notifications I'm participating in</label>
        </div>
        <div class="checkbox-row">
            <input type="checkbox" id="show_desktop_notifications">
            <label for="show_desktop_notifications">Show desktop notifications</label>
        </div>
        <div class="checkbox-row">
            <input type="checkbox" id="play_sound">
            <label for="play_sound">Play a sound on new notifications</label>
        </div>
        <div class="checkbox-row">
            <input type="checkbox" id="reuse_existing_tab">
            <label for="reuse_existing_tab">Reuse an existing notifications tab</label>
        </div>

        <div class="actions">
            <button type="submit" class="primary">Save</button>
            <button type="button" id="test-connection">Test Connection</button>
            <span id="status"></span>
        </div>
    </form>
    <script type="module" src="options.js"></script>
</body>
</html>
'#####
        $options_html | save -f ([$out_dir, "options.html"] | path join)

        let options_js = r#####'const SETTINGS_KEY = 'settings';

const DEFAULTS = {
    token: '',
    api_url: 'https://api.github.com',
    root_url: 'https://github.com',
    only_participating: false,
    poll_interval_mins: 1,
    show_desktop_notifications: true,
    play_sound: false,
    reuse_existing_tab: true,
    repo_filter_mode: 'None',
    repo_filter_list: [],
};

const form = document.getElementById('settings-form');
const statusEl = document.getElementById('status');

function setStatus(message, isError) {
    statusEl.textContent = message;
    statusEl.style.color = isError ? '#cf222e' : '#57606a';
}

function applySettingsToForm(settings) {
    form.token.value = settings.token;
    form.api_url.value = settings.api_url;
    form.root_url.value = settings.root_url;
    form.poll_interval_mins.value = settings.poll_interval_mins;
    form.repo_filter_mode.value = settings.repo_filter_mode;
    form.repo_filter_list.value = settings.repo_filter_list.join('\n');
    form.only_participating.checked = settings.only_participating;
    form.show_desktop_notifications.checked = settings.show_desktop_notifications;
    form.play_sound.checked = settings.play_sound;
    form.reuse_existing_tab.checked = settings.reuse_existing_tab;
}

function readSettingsFromForm() {
    const repoList = form.repo_filter_list.value
        .split(/[\n,]/)
        .map((s) => s.trim())
        .filter((s) => s.length > 0);

    return {
        token: form.token.value.trim(),
        api_url: form.api_url.value.trim() || DEFAULTS.api_url,
        root_url: form.root_url.value.trim() || DEFAULTS.root_url,
        only_participating: form.only_participating.checked,
        poll_interval_mins: Math.max(1, parseInt(form.poll_interval_mins.value, 10) || DEFAULTS.poll_interval_mins),
        show_desktop_notifications: form.show_desktop_notifications.checked,
        play_sound: form.play_sound.checked,
        reuse_existing_tab: form.reuse_existing_tab.checked,
        repo_filter_mode: form.repo_filter_mode.value,
        repo_filter_list: repoList,
    };
}

async function load() {
    const data = await chrome.storage.local.get(SETTINGS_KEY);
    applySettingsToForm({ ...DEFAULTS, ...(data[SETTINGS_KEY] || {}) });
}

form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const settings = readSettingsFromForm();
    await chrome.storage.local.set({ [SETTINGS_KEY]: settings });
    setStatus('Settings saved.', false);
});

document.getElementById('test-connection').addEventListener('click', async () => {
    const settings = readSettingsFromForm();
    if (!settings.token) {
        setStatus('Enter a token first.', true);
        return;
    }
    setStatus('Testing...', false);
    try {
        const res = await fetch(`${settings.api_url.replace(/\/+$/, '')}/notifications?per_page=1`, {
            headers: {
                'Authorization': `Bearer ${settings.token}`,
                'X-GitHub-Api-Version': '2022-11-28',
                'Accept': 'application/vnd.github+json',
            },
        });
        if (res.ok) {
            setStatus('Connection successful.', false);
        } else {
            setStatus(`Connection failed: HTTP ${res.status}`, true);
        }
    } catch (e) {
        setStatus(`Connection failed: ${e.message}`, true);
    }
});

load().catch((e) => setStatus(`Failed to load settings: ${e.message}`, true));
'#####
        $options_js | save -f ([$out_dir, "options.js"] | path join)

        # Archive package
        let zip_file = ([$dist_dir, $"github-notified-($t).zip"] | path join)
        make_zip $out_dir $zip_file
        print $"==> Created package archive: ($zip_file)"
    }

    print "==> Extension packaging finished successfully!"
}

# Entrypoint when invoked directly as a script
def main [
    --target: string = "all",
    --release
] {
    package_extension --target $target --release=$release
}
