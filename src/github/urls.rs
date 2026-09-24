use crate::github::models::NotificationThread;

/// Translates a GitHub API URL or notification thread into a browser-openable Web URL.
pub fn notification_to_web_url(thread: &NotificationThread, root_url: &str) -> String {
    let clean_root = root_url.trim_end_matches('/');

    // 1. Try parsing from subject.url (or latest_comment_url)
    if let Some(api_url) = &thread.subject.url {
        if let Some(web_url) = api_url_to_web_url(api_url, clean_root) {
            return append_comment_hash(web_url, thread.subject.latest_comment_url.as_deref());
        }
    }

    // 2. Fallback to repository html_url if available
    if !thread.repository.html_url.is_empty() {
        return thread.repository.html_url.clone();
    }

    // 3. Fallback to repository name
    if !thread.repository.full_name.is_empty() {
        return format!("{clean_root}/{}", thread.repository.full_name);
    }

    // 4. Default fallback to notifications page
    format!("{clean_root}/notifications")
}

/// Convert an API URL like "https://api.github.com/repos/owner/repo/pulls/12" to "https://github.com/owner/repo/pull/12"
pub fn api_url_to_web_url(api_url: &str, clean_root: &str) -> Option<String> {
    // Look for "/repos/" in the path
    let repos_idx = api_url.find("/repos/")?;
    let path_after_repos = &api_url[repos_idx + "/repos/".len()..];
    let parts: Vec<&str> = path_after_repos.split('/').collect();

    if parts.len() < 2 {
        return None;
    }

    let owner = parts[0];
    let repo = parts[1];

    if parts.len() == 2 {
        return Some(format!("{clean_root}/{owner}/{repo}"));
    }

    let resource_type = parts[2];

    match resource_type {
        "pulls" => {
            if parts.len() >= 4 {
                let number = parts[3];
                Some(format!("{clean_root}/{owner}/{repo}/pull/{number}"))
            } else {
                Some(format!("{clean_root}/{owner}/{repo}/pulls"))
            }
        }
        "issues" => {
            if parts.len() >= 4 {
                let number = parts[3];
                Some(format!("{clean_root}/{owner}/{repo}/issues/{number}"))
            } else {
                Some(format!("{clean_root}/{owner}/{repo}/issues"))
            }
        }
        "commits" => {
            if parts.len() >= 4 {
                let sha = parts[3];
                Some(format!("{clean_root}/{owner}/{repo}/commit/{sha}"))
            } else {
                Some(format!("{clean_root}/{owner}/{repo}/commits"))
            }
        }
        "releases" => Some(format!("{clean_root}/{owner}/{repo}/releases")),
        "discussions" => {
            if parts.len() >= 4 {
                let number = parts[3];
                Some(format!("{clean_root}/{owner}/{repo}/discussions/{number}"))
            } else {
                Some(format!("{clean_root}/{owner}/{repo}/discussions"))
            }
        }
        other => {
            let remainder = parts[2..].join("/");
            Some(format!("{clean_root}/{owner}/{repo}/{other}/{remainder}"))
        }
    }
}

/// Append comment fragment anchor if latest_comment_url is available
fn append_comment_hash(base_url: String, latest_comment_url: Option<&str>) -> String {
    let Some(comment_url) = latest_comment_url else {
        return base_url;
    };

    if comment_url.contains("/issues/comments/") {
        if let Some(comment_id) = comment_url.split("/issues/comments/").nth(1) {
            return format!("{base_url}#issuecomment-{comment_id}");
        }
    } else if comment_url.contains("/pulls/comments/") {
        if let Some(comment_id) = comment_url.split("/pulls/comments/").nth(1) {
            return format!("{base_url}#discussion_r{comment_id}");
        }
    }

    base_url
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::models::{Repository, Subject};

    #[test]
    fn test_issue_url_conversion() {
        let api_url = "https://api.github.com/repos/rust-lang/rust/issues/12345";
        let web_url = api_url_to_web_url(api_url, "https://github.com").unwrap();
        assert_eq!(web_url, "https://github.com/rust-lang/rust/issues/12345");
    }

    #[test]
    fn test_pull_request_url_conversion() {
        let api_url = "https://api.github.com/repos/0xsouravm/oxichrome/pulls/8";
        let web_url = api_url_to_web_url(api_url, "https://github.com").unwrap();
        // Notice: "pulls" -> "pull"
        assert_eq!(web_url, "https://github.com/0xsouravm/oxichrome/pull/8");
    }

    #[test]
    fn test_commit_url_conversion() {
        let api_url = "https://api.github.com/repos/rust-lang/cargo/commits/abcdef123456";
        let web_url = api_url_to_web_url(api_url, "https://github.com").unwrap();
        assert_eq!(
            web_url,
            "https://github.com/rust-lang/cargo/commit/abcdef123456"
        );
    }

    #[test]
    fn test_enterprise_root_url() {
        let api_url = "https://github.internal.mycorp.com/api/v3/repos/team/repo/issues/99";
        let web_url = api_url_to_web_url(api_url, "https://github.internal.mycorp.com").unwrap();
        assert_eq!(
            web_url,
            "https://github.internal.mycorp.com/team/repo/issues/99"
        );
    }

    #[test]
    fn test_thread_with_latest_comment() {
        let thread = NotificationThread {
            id: "101".to_string(),
            unread: true,
            reason: "subscribed".to_string(),
            updated_at: "2026-09-23T00:00:00Z".to_string(),
            last_read_at: None,
            subject: Subject {
                title: "Bug report".to_string(),
                url: Some("https://api.github.com/repos/owner/repo/issues/50".to_string()),
                latest_comment_url: Some(
                    "https://api.github.com/repos/owner/repo/issues/comments/777888".to_string(),
                ),
                type_name: "Issue".to_string(),
            },
            repository: Repository {
                id: 1,
                name: "repo".to_string(),
                full_name: "owner/repo".to_string(),
                private: false,
                html_url: "https://github.com/owner/repo".to_string(),
                owner: None,
            },
            url: "https://api.github.com/notifications/threads/101".to_string(),
        };

        let web_url = notification_to_web_url(&thread, "https://github.com");
        assert_eq!(
            web_url,
            "https://github.com/owner/repo/issues/50#issuecomment-777888"
        );
    }
}
