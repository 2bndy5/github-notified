#[cfg(target_arch = "wasm32")]
use crate::github::models::{GitHubUser, NotificationThread};

pub const GITHUB_API_VERSION: &str = "2022-11-28";
pub const USER_AGENT: &str = "github-notified-rust";

#[derive(Debug, Clone)]
pub struct GitHubClient {
    token: String,
    api_url: String,
}

impl GitHubClient {
    pub fn new(token: impl Into<String>, api_url: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_url: api_url.into(),
        }
    }

    /// Generates standard headers required by GitHub REST API
    pub fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token.trim())
    }

    /// Construct notification query URL
    pub fn build_notifications_url(&self, only_participating: bool, since: Option<&str>) -> String {
        let base = self.api_url.trim_end_matches('/');
        let mut url = format!("{base}/notifications?per_page=100");

        if only_participating {
            url.push_str("&participating=true");
        }

        if let Some(s) = since {
            if !s.is_empty() {
                url.push_str(&format!("&since={s}"));
            }
        }

        url
    }

    /// Fetch notifications from GitHub REST API
    #[cfg(target_arch = "wasm32")]
    pub async fn fetch_notifications(
        &self,
        only_participating: bool,
        since: Option<&str>,
    ) -> Result<Vec<NotificationThread>, String> {
        let url = self.build_notifications_url(only_participating, since);

        let response = gloo_net::http::Request::get(&url)
            .header("Authorization", &self.auth_header_value())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {e}"))?;

        let status = response.status();
        if status == 401 {
            return Err(
                "Unauthorized: Invalid or expired GitHub Personal Access Token".to_string(),
            );
        }
        if status == 403 {
            return Err(
                "Forbidden: Rate limit reached or token lacks 'Notifications' permission"
                    .to_string(),
            );
        }
        if !response.ok() {
            return Err(format!("GitHub API error (HTTP {status})"));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {e}"))?;

        let notifications = serde_json::from_str::<Vec<NotificationThread>>(&body)
            .map_err(|e| format!("Failed to parse notifications JSON: {e}"))?;

        Ok(notifications)
    }

    /// Validate the token by querying /notifications?per_page=1
    /// (Requires a classic PAT with the `notifications` scope)
    #[cfg(target_arch = "wasm32")]
    pub async fn validate_token(&self) -> Result<String, String> {
        if self.token.trim().is_empty() {
            return Err("Token is empty".to_string());
        }

        let base = self.api_url.trim_end_matches('/');
        let test_url = format!("{base}/notifications?per_page=1");

        let response = gloo_net::http::Request::get(&test_url)
            .header("Authorization", &self.auth_header_value())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("Network connection failed: {e}"))?;

        let status = response.status();
        if status == 200 {
            // Optional: try to fetch user name from /user if token has user scope
            if let Ok(user) = self.fetch_current_user().await {
                return Ok(format!("Connected as @{}", user.login));
            }
            Ok("Token is valid! Connected to GitHub notifications.".to_string())
        } else if status == 401 {
            Err("Unauthorized: Invalid or expired token.".to_string())
        } else if status == 403 {
            Err("Forbidden: Token is missing 'Notifications: Read-only' permission or rate limited.".to_string())
        } else {
            Err(format!("Error validating token (HTTP {status})"))
        }
    }

    /// Try fetching current user details (if token has Metadata/User scope)
    #[cfg(target_arch = "wasm32")]
    pub async fn fetch_current_user(&self) -> Result<GitHubUser, String> {
        let base = self.api_url.trim_end_matches('/');
        let user_url = format!("{base}/user");

        let response = gloo_net::http::Request::get(&user_url)
            .header("Authorization", &self.auth_header_value())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("User request failed: {e}"))?;

        if !response.ok() {
            return Err(format!("HTTP error {}", response.status()));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read user body: {e}"))?;

        serde_json::from_str::<GitHubUser>(&body).map_err(|e| format!("JSON error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header_bearer() {
        let client =
            GitHubClient::new("ghp_classicToken11XYZ_testSecret", "https://api.github.com");
        assert_eq!(
            client.auth_header_value(),
            "Bearer ghp_classicToken11XYZ_testSecret"
        );

        let padded_client = GitHubClient::new(" ghp_classic123 ", "https://api.github.com");
        assert_eq!(padded_client.auth_header_value(), "Bearer ghp_classic123");
    }

    #[test]
    fn test_build_notifications_url() {
        let client = GitHubClient::new("token", "https://api.github.com");
        let url = client.build_notifications_url(false, None);
        assert_eq!(url, "https://api.github.com/notifications?per_page=100");

        let url_part = client.build_notifications_url(true, Some("2026-09-23T12:00:00Z"));
        assert_eq!(
            url_part,
            "https://api.github.com/notifications?per_page=100&participating=true&since=2026-09-23T12:00:00Z"
        );
    }
}
