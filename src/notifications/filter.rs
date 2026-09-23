use crate::config::{Config, RepoFilterMode};
use crate::github::models::NotificationThread;

/// Filters notification threads according to repository whitelist/blacklist rules.
pub fn filter_notifications<'a>(
    notifications: &'a [NotificationThread],
    config: &Config,
) -> Vec<&'a NotificationThread> {
    match config.repo_filter_mode {
        RepoFilterMode::None => notifications.iter().collect(),
        RepoFilterMode::Whitelist => {
            let filter_set: Vec<String> = config
                .repo_filter_list
                .iter()
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            if filter_set.is_empty() {
                return notifications.iter().collect();
            }

            notifications
                .iter()
                .filter(|n| {
                    let full_name = n.repository.full_name.trim().to_lowercase();
                    filter_set.contains(&full_name)
                })
                .collect()
        }
        RepoFilterMode::Blacklist => {
            let filter_set: Vec<String> = config
                .repo_filter_list
                .iter()
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            notifications
                .iter()
                .filter(|n| {
                    let full_name = n.repository.full_name.trim().to_lowercase();
                    !filter_set.contains(&full_name)
                })
                .collect()
        }
    }
}

/// Computes the newly arrived notification IDs that were not present in previous cache
pub fn compute_new_notifications<'a>(
    current: &'a [&NotificationThread],
    previous_ids: &[String],
) -> Vec<&'a NotificationThread> {
    current
        .iter()
        .filter(|item| !previous_ids.iter().any(|prev_id| prev_id == &item.id))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::models::{Repository, Subject};

    fn make_thread(id: &str, repo_name: &str) -> NotificationThread {
        NotificationThread {
            id: id.to_string(),
            unread: true,
            reason: "mention".to_string(),
            updated_at: "2026-09-23T12:00:00Z".to_string(),
            last_read_at: None,
            subject: Subject {
                title: format!("Title {id}"),
                url: None,
                latest_comment_url: None,
                type_name: "Issue".to_string(),
            },
            repository: Repository {
                id: 1,
                name: repo_name.to_string(),
                full_name: repo_name.to_string(),
                private: false,
                html_url: format!("https://github.com/{repo_name}"),
                owner: None,
            },
            url: format!("https://api.github.com/notifications/threads/{id}"),
        }
    }

    #[test]
    fn test_whitelist_filter() {
        let threads = vec![
            make_thread("1", "rust-lang/rust"),
            make_thread("2", "0xsouravm/oxichrome"),
            make_thread("3", "sindresorhus/notifier-for-github"),
        ];

        let config = Config {
            repo_filter_mode: RepoFilterMode::Whitelist,
            repo_filter_list: vec![
                "rust-lang/rust".to_string(),
                "0xsouravm/oxichrome".to_string(),
            ],
            ..Default::default()
        };

        let filtered = filter_notifications(&threads, &config);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "1");
        assert_eq!(filtered[1].id, "2");
    }

    #[test]
    fn test_blacklist_filter() {
        let threads = vec![
            make_thread("1", "rust-lang/rust"),
            make_thread("2", "spammer/noise"),
        ];

        let config = Config {
            repo_filter_mode: RepoFilterMode::Blacklist,
            repo_filter_list: vec!["spammer/noise".to_string()],
            ..Default::default()
        };

        let filtered = filter_notifications(&threads, &config);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].repository.full_name, "rust-lang/rust");
    }

    #[test]
    fn test_compute_new_notifications() {
        let thread1 = make_thread("1", "a/b");
        let thread2 = make_thread("2", "c/d");
        let current = vec![&thread1, &thread2];

        let prev_ids = vec!["1".to_string()];
        let newly_arrived = compute_new_notifications(&current, &prev_ids);

        assert_eq!(newly_arrived.len(), 1);
        assert_eq!(newly_arrived[0].id, "2");
    }
}
