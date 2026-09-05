use std::path::Path;
use std::time::Duration;

use askama::Template;
use axum::extract::Extension;
use axum::response::{Html, IntoResponse};

use crate::config::{Config, SyncCheck};
use crate::vault::{VaultReader, count_notes_recursive};

#[derive(Template)]
#[template(path = "status.html")]
struct StatusTemplate {
    note_count: usize,
    folder_count: usize,
    uptime: String,
    last_modified: String,
    vault_path: String,
    sync_status: &'static str,
    auth_status: &'static str,
    version: String,
}

#[derive(Template)]
#[template(path = "status_stats.html")]
struct StatsTemplate {
    note_count: usize,
    folder_count: usize,
    uptime: String,
    vault_path: String,
    last_modified: String,
    sync_status: &'static str,
    auth_status: &'static str,
}

pub async fn status_page(Extension(cfg): Extension<Config>) -> impl IntoResponse {
    let vault = VaultReader::new(std::path::PathBuf::from(&cfg.vault_dir));
    let (note_count, folder_count) = vault_stats(vault.root());

    let template = StatusTemplate {
        note_count,
        folder_count,
        last_modified: vault.last_modified().map_or_else(
            || "Never".into(),
            |e| format_duration(e.elapsed().unwrap_or(Duration::ZERO)),
        ),
        uptime: format_duration(cfg.started_at.elapsed()),
        vault_path: cfg.vault_dir.clone(),
        sync_status: determine_sync_status(&cfg, &vault),
        auth_status: if cfg.auth_enabled {
            "OIDC"
        } else {
            "Disabled (dev)"
        },
        version: cfg.version,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|_| "<p>Template error</p>".to_string()),
    )
}

pub async fn status_stats(Extension(cfg): Extension<Config>) -> impl IntoResponse {
    let vault = VaultReader::new(std::path::PathBuf::from(&cfg.vault_dir));
    let (note_count, folder_count) = vault_stats(vault.root());

    let template = StatsTemplate {
        note_count,
        folder_count,
        uptime: format_duration(cfg.started_at.elapsed()),
        last_modified: vault.last_modified().map_or_else(
            || "Never".into(),
            |e| format_duration(e.elapsed().unwrap_or(Duration::ZERO)),
        ),
        vault_path: cfg.vault_dir.clone(),
        sync_status: determine_sync_status(&cfg, &vault),
        auth_status: if cfg.auth_enabled {
            "OIDC"
        } else {
            "Disabled (dev)"
        },
    };

    Html(template.render().unwrap_or_else(|_| {
        "<p>Could not construct current status. Check the Logs</p>".to_string()
    }))
}

fn determine_sync_status(cfg: &Config, vault: &VaultReader) -> &'static str {
    match cfg.sync_check {
        SyncCheck::File => vault.last_modified().map_or("Unknown", |t| {
            let age = t.elapsed().unwrap_or(Duration::ZERO);
            if age < Duration::from_mins(5) {
                "Running"
            } else {
                "Stale"
            }
        }),
        SyncCheck::Heartbeat => vault.sync_heartbeat().map_or("Unknown", |age| {
            if age < Duration::from_mins(5) {
                "Running"
            } else {
                "Stale"
            }
        }),
        SyncCheck::None | SyncCheck::Kubernetes => "Unknown",
    }
}

fn vault_stats(root: &Path) -> (usize, usize) {
    let mut folders = Vec::new();
    let _ = count_notes_recursive(root, "", &mut folders);

    let note_count: usize = folders.iter().map(|(_, count)| count).sum();
    let folder_count = folders.len();

    (note_count, folder_count)
}

fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    match (days, hours, mins) {
        (0, 0, 0) => "just now".to_string(),
        (0, 0, m) => format!("{m}m"),
        (0, h, m) => format!("{h}h {m}m"),
        (d, h, _) => format!("{d}d {h}h"),
    }
}
