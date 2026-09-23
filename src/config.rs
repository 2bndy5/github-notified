use serde::{Deserialize, Serialize};

pub const DEFAULT_API_URL: &str = "https://api.github.com";
pub const DEFAULT_ROOT_URL: &str = "https://github.com";
pub const STORAGE_KEY_SETTINGS: &str = "settings";
pub const STORAGE_KEY_NOTIFICATIONS_CACHE: &str = "notifications_cache";
pub const STORAGE_KEY_LAST_ERROR: &str = "last_error";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RepoFilterMode {
    #[default]
    None,
    Whitelist,
    Blacklist,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// GitHub Personal Access Token (Fine-grained or Classic)
    pub token: String,
    /// GitHub API URL (supports GitHub Enterprise)
    pub api_url: String,
    /// GitHub Web Root URL (supports GitHub Enterprise)
    pub root_url: String,
    /// Only show notifications for issues/PRs the user is participating in
    pub only_participating: bool,
    /// Periodic checking interval in minutes (minimum 1 minute per Chrome MV3)
    pub poll_interval_mins: u32,
    /// Show desktop notification on new unread notifications
    pub show_desktop_notifications: bool,
    /// Play chime/sound on new notification
    pub play_sound: bool,
    /// Reuse existing notification tab when toolbar icon is clicked
    pub reuse_existing_tab: bool,
    /// Mode for repository filtering (None, Whitelist, Blacklist)
    pub repo_filter_mode: RepoFilterMode,
    /// Comma/newline separated repository full names ("owner/repo")
    pub repo_filter_list: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: String::new(),
            api_url: DEFAULT_API_URL.to_string(),
            root_url: DEFAULT_ROOT_URL.to_string(),
            only_participating: false,
            poll_interval_mins: 1,
            show_desktop_notifications: true,
            play_sound: false,
            reuse_existing_tab: true,
            repo_filter_mode: RepoFilterMode::None,
            repo_filter_list: Vec::new(),
        }
    }
}

impl Config {
    /// Returns the notifications Web URL (e.g. "https://github.com/notifications")
    pub fn notifications_url(&self) -> String {
        let base = self.root_url.trim_end_matches('/');
        if self.only_participating {
            format!("{base}/notifications/participating")
        } else {
            format!("{base}/notifications")
        }
    }

    /// Check if a valid token is set
    pub fn has_token(&self) -> bool {
        !self.token.trim().is_empty()
    }

    /// Check if the token looks like a modern GitHub fine-grained PAT
    pub fn is_fine_grained_token(&self) -> bool {
        self.token.trim().starts_with("github_pat_")
    }

    /// Load config from browser local storage (or default on error/missing)
    #[cfg(target_arch = "wasm32")]
    pub async fn load() -> Self {
        match oxichrome::storage::get::<Config>(STORAGE_KEY_SETTINGS).await {
            Ok(Some(cfg)) => cfg,
            _ => Config::default(),
        }
    }

    /// Load default on non-wasm targets (e.g. unit tests)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn load() -> Self {
        Config::default()
    }

    /// Save config to browser local storage
    #[cfg(target_arch = "wasm32")]
    pub async fn save(&self) -> Result<(), oxichrome::OxichromeError> {
        oxichrome::storage::set(STORAGE_KEY_SETTINGS, self).await
    }

    /// No-op on non-wasm targets
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn save(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.api_url, "https://api.github.com");
        assert_eq!(config.root_url, "https://github.com");
        assert_eq!(config.poll_interval_mins, 1);
        assert!(!config.has_token());
        assert!(!config.only_participating);
        assert!(config.show_desktop_notifications);
        assert_eq!(
            config.notifications_url(),
            "https://github.com/notifications"
        );
    }

    #[test]
    fn test_token_type_detection() {
        let config = Config {
            token: "github_pat_11AAAAAA_BBBBBBBBBBBB".to_string(),
            ..Default::default()
        };
        assert!(config.has_token());
        assert!(config.is_fine_grained_token());

        let classic_config = Config {
            token: "ghp_classicToken123456789".to_string(),
            ..Default::default()
        };
        assert!(classic_config.has_token());
        assert!(!classic_config.is_fine_grained_token());
    }

    #[test]
    fn test_custom_root_url() {
        let config = Config {
            root_url: "https://ghe.company.internal/".to_string(),
            ..Default::default()
        };
        assert_eq!(
            config.notifications_url(),
            "https://ghe.company.internal/notifications"
        );

        let participating_config = Config {
            root_url: "https://ghe.company.internal/".to_string(),
            only_participating: true,
            ..Default::default()
        };
        assert_eq!(
            participating_config.notifications_url(),
            "https://ghe.company.internal/notifications/participating"
        );
    }
}
