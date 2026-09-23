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

    let dist_dir = "dist"
    let targets = if $target == "all" { ["chromium", "firefox"] } else { [$target] }

    let has_wasm_bindgen = (which wasm-bindgen | is-not-empty)

    for t in $targets {
        let out_dir = ([$dist_dir, $t] | path join)
        let wasm_out = ([$out_dir, "wasm"] | path join)

        if ($out_dir | path exists) {
            rm -rf $out_dir
        }
        mkdir $wasm_out

        print $"==> Processing ($t) extension distribution at ($out_dir)..."

        if $has_wasm_bindgen {
            wasm-bindgen --target web --out-dir $wasm_out --out-name "github_notified" $wasm_file
        } else {
            cp $wasm_file ([$wasm_out, "github_notified_bg.wasm"] | path join)
        }

        # Copy static assets (icons)
        if ("static" | path exists) {
            for asset in (ls static) {
                cp $asset.name $out_dir
            }
        }

        # Generate Manifest V3
        let base_manifest = {
            manifest_version: 3,
            name: "GitHub Notified",
            version: "0.1.0",
            description: "Browser extension for GitHub notifications with fine-grained PAT support",
            permissions: ["storage", "alarms", "notifications", "tabs"],
            host_permissions: ["https://api.github.com/*", "https://github.com/*"],
            icons: {
                "16": "icon-16.png",
                "32": "icon-32.png",
                "48": "icon-48.png",
                "128": "icon-128.png"
            },
            action: {
                default_popup: "popup.html",
                default_icon: {
                    "16": "icon-16.png",
                    "32": "icon-32.png",
                    "48": "icon-48.png",
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
                        id: "github-notified@oxichrome.dev"
                    }
                }
        }

        $manifest | to json -i 2 | save -f ([$out_dir, "manifest.json"] | path join)

        # background.js shim
        let bg_js = "// Auto-generated shim for github-notified
import init, { __oxichrome_bg_start, __oxichrome_register_handle_install } from './wasm/github_notified.js';

async function start() {
    try {
        await init();
        if (typeof __oxichrome_register_handle_install === 'function') {
            __oxichrome_register_handle_install();
        }
        if (typeof __oxichrome_bg_start === 'function') {
            __oxichrome_bg_start();
        }
        console.log('[oxichrome] GitHub Notified background worker initialized.');
    } catch (e) {
        console.error('[oxichrome] Failed to initialize WASM:', e);
    }
}

start();
"
        $bg_js | save -f ([$out_dir, "background.js"] | path join)

        # popup.html & popup.js
        let popup_html = "<!DOCTYPE html>
<html>
<head>
    <meta charset=\"UTF-8\">
    <title>GitHub Notified</title>
    <style>html, body { margin: 0; padding: 0; width: 360px; min-height: 200px; }</style>
</head>
<body>
    <script type=\"module\" src=\"popup.js\"></script>
</body>
</html>
"
        $popup_html | save -f ([$out_dir, "popup.html"] | path join)

        let popup_js = "import init, { __oxichrome_mount_popup } from './wasm/github_notified.js';

try {
    await init();
    if (typeof __oxichrome_mount_popup === 'function') {
        __oxichrome_mount_popup();
    }
} catch (e) {
    console.error('[oxichrome] Popup init failed:', e);
    document.body.textContent = 'Error loading popup: ' + e.message;
}
"
        $popup_js | save -f ([$out_dir, "popup.js"] | path join)

        # options.html & options.js
        let options_html = "<!DOCTYPE html>
<html>
<head>
    <meta charset=\"UTF-8\">
    <title>GitHub Notified - Settings</title>
    <style>html, body { margin: 0; padding: 0; background: #f6f8fa; }</style>
</head>
<body>
    <script type=\"module\" src=\"options.js\"></script>
</body>
</html>
"
        $options_html | save -f ([$out_dir, "options.html"] | path join)

        let options_js = "import init, { __oxichrome_mount_options } from './wasm/github_notified.js';

try {
    await init();
    if (typeof __oxichrome_mount_options === 'function') {
        __oxichrome_mount_options();
    }
} catch (e) {
    console.error('[oxichrome] Options init failed:', e);
    document.body.textContent = 'Error loading options: ' + e.message;
}
"
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
