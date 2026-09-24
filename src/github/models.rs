use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationThread {
    pub id: String,
    pub unread: bool,
    pub reason: String,
    pub updated_at: String,
    #[serde(default)]
    pub last_read_at: Option<String>,
    pub subject: Subject,
    pub repository: Repository,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub title: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub latest_comment_url: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub owner: Option<Owner>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owner {
    pub login: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationsCache {
    pub count: usize,
    pub last_checked: String,
    pub notifications: Vec<NotificationSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationSummary {
    pub id: String,
    pub repo_full_name: String,
    pub title: String,
    pub type_name: String,
    pub reason: String,
    pub updated_at: String,
    pub web_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_notification() {
        let json_data = r#"{
            "id": "123456789",
            "unread": true,
            "reason": "mention",
            "updated_at": "2026-09-23T12:00:00Z",
            "last_read_at": null,
            "subject": {
                "title": "Fix memory leak in parser",
                "url": "https://api.github.com/repos/0xsouravm/oxichrome/issues/42",
                "latest_comment_url": "https://api.github.com/repos/0xsouravm/oxichrome/issues/comments/999",
                "type": "Issue"
            },
            "repository": {
                "id": 987654,
                "name": "oxichrome",
                "full_name": "0xsouravm/oxichrome",
                "private": false,
                "html_url": "https://github.com/0xsouravm/oxichrome"
            },
            "url": "https://api.github.com/notifications/threads/123456789"
        }"#;

        let thread: NotificationThread = serde_json::from_str(json_data).unwrap();
        assert_eq!(thread.id, "123456789");
        assert!(thread.unread);
        assert_eq!(thread.reason, "mention");
        assert_eq!(thread.subject.title, "Fix memory leak in parser");
        assert_eq!(thread.subject.type_name, "Issue");
        assert_eq!(thread.repository.full_name, "0xsouravm/oxichrome");
    }
}
