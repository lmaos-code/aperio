use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub frontmatter: Option<Frontmatter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub path: PathBuf,
    pub title: String,
    pub tags: Vec<String>,
    pub modified: SystemTime,
}

#[derive(Clone)]
pub struct VaultReader {
    root: PathBuf,
}

impl VaultReader {
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list_notes(&self, folder: Option<&str>) -> Result<Vec<NoteSummary>, std::io::Error> {
        let search_dir = folder.map_or_else(|| self.root.clone(), |f| self.root.join(f));

        let mut notes = Vec::new();
        self.walk_dir(&search_dir, &mut notes)?;
        notes.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(notes)
    }

    fn walk_dir(&self, dir: &Path, notes: &mut Vec<NoteSummary>) -> Result<(), std::io::Error> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.starts_with('.') {
                    debug!("Skipping hidden directory: {dir_name}");
                    continue;
                }
                self.walk_dir(&path, notes)?;
            } else if path.extension().is_some_and(|e| e == "md")
                && let Ok(note) = self.read_note_summary(&path)
            {
                notes.push(note);
            }
        }
        Ok(())
    }

    pub fn read_note(&self, relative_path: &str) -> Result<Note, std::io::Error> {
        let path = self.root.join(relative_path);
        let content = std::fs::read_to_string(&path)?;
        let (frontmatter, body) = parse_frontmatter(&content);
        let title = frontmatter
            .as_ref()
            .and_then(|f| f.title.clone())
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

        Ok(Note {
            path: path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf(),
            title,
            content: body,
            frontmatter,
        })
    }

    fn read_note_summary(&self, path: &Path) -> Result<NoteSummary, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let (frontmatter, _) = parse_frontmatter(&content);
        let title = frontmatter
            .as_ref()
            .and_then(|f| f.title.clone())
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
        let tags = frontmatter
            .as_ref()
            .map(|f| f.tags.clone())
            .unwrap_or_default();

        let relative = path.strip_prefix(&self.root).unwrap_or(path).to_path_buf();

        let modified = std::fs::metadata(path)?.modified()?;

        Ok(NoteSummary {
            path: relative,
            title,
            tags,
            modified,
        })
    }

    pub fn write_note(&self, relative_path: &str, content: &str) -> Result<Note, std::io::Error> {
        let path = self.root.join(relative_path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, content)?;

        let (frontmatter, body) = parse_frontmatter(content);
        let title = frontmatter
            .as_ref()
            .and_then(|f| f.title.clone())
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

        Ok(Note {
            path: path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf(),
            title,
            content: body,
            frontmatter,
        })
    }

    pub fn delete_note(&self, relative_path: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.root.join(relative_path);

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let trash_dir = self.root.join(".trash");
        std::fs::create_dir_all(&trash_dir)?;

        let mut trash_path = trash_dir.join(&file_name);
        let mut counter = 1usize;
        while trash_path.exists() {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            trash_path = trash_dir.join(format!("{stem}_{counter}{ext}"));
            counter = counter.saturating_add(1);
        }

        std::fs::rename(&path, &trash_path)?;

        Ok(trash_path
            .strip_prefix(&self.root)
            .unwrap_or(&trash_path)
            .to_path_buf())
    }

    pub fn note_metadata(&self, relative_path: &str) -> Result<NoteSummary, std::io::Error> {
        let path = self.root.join(relative_path);
        self.read_note_summary(&path)
    }

    pub fn last_modified(&self) -> Option<SystemTime> {
        let mut notes = Vec::new();
        self.walk_dir(&self.root, &mut notes).ok()?;

        let pivot = notes.first()?;
        Some(notes.iter().fold(pivot.modified, |acc, elem| {
            if elem.modified.gt(&acc) {
                return elem.modified;
            }
            acc
        }))
    }

    pub fn sync_heartbeat(&self) -> Option<Duration> {
        let heartbeat_path = self.root.join(".sync-heartbeat");
        let content = std::fs::read_to_string(heartbeat_path).ok()?;
        let trimmed = content.trim();

        let ts = parse_iso8601(trimmed)?;
        let now = SystemTime::now();
        let elapsed = now.duration_since(ts).ok()?;
        Some(elapsed)
    }
}

fn parse_frontmatter(content: &str) -> (Option<Frontmatter>, String) {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return (None, content.to_string());
    }

    let Some(after_first) = content.get(3..) else {
        return (None, content.to_string());
    };

    after_first
        .find("\n---")
        .and_then(|end_idx| {
            let yaml_str = after_first.get(..end_idx)?;
            let offset = end_idx.checked_add(4)?;
            let body_start = after_first.get(offset..)?;
            let body = body_start.trim().to_string();

            serde_yaml::from_str::<Frontmatter>(yaml_str)
                .ok()
                .map(|fm| (Some(fm), body))
        })
        .unwrap_or_else(|| (None, content.to_string()))
}

pub fn count_notes_recursive(
    dir: &Path,
    prefix: &str,
    folders: &mut Vec<(String, usize)>,
) -> Result<(), std::io::Error> {
    if !dir.exists() {
        return Ok(());
    }

    let mut note_count = 0usize;
    let mut subdirs = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            subdirs.push((name.to_string(), path));
        } else if path.extension().is_some_and(|e| e == "md") {
            note_count = note_count.saturating_add(1);
        }
    }

    if note_count > 0 || !subdirs.is_empty() {
        folders.push((prefix.to_string(), note_count));
    }

    for (dir_name, dir_path) in subdirs {
        let sub_prefix = if prefix.is_empty() {
            dir_name.clone()
        } else {
            format!("{prefix}/{dir_name}")
        };
        count_notes_recursive(&dir_path, &sub_prefix, folders)?;
    }

    Ok(())
}

fn parse_iso8601(s: &str) -> Option<SystemTime> {
    let s = s.strip_suffix('Z').unwrap_or(s);

    let (date_part, time_part) = s.split_once('T')?;

    let mut date_components = date_part.split('-').filter_map(|p| p.parse::<u32>().ok());
    let year = date_components.next()?;
    let month = date_components.next()?;
    let day = date_components.next()?;

    let time_str = time_part.strip_suffix('Z').unwrap_or(time_part);
    let mut time_components = time_str
        .split([':', '.'])
        .filter_map(|p| p.parse::<u32>().ok());
    let hours = time_components.next()?;
    let minutes = time_components.next().unwrap_or(0);
    let seconds = time_components.next().unwrap_or(0);

    let days_from_epoch = days_since_epoch(year, month, day)?;
    let total_secs = days_from_epoch
        .checked_mul(86400)?
        .checked_add(u64::from(hours).checked_mul(3600)?)?
        .checked_add(u64::from(minutes).checked_mul(60)?)?
        .checked_add(u64::from(seconds))?;

    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(total_secs))
}

fn days_since_epoch(year: u32, month: u32, day: u32) -> Option<u64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let y = i64::from(year);
    let m = i64::from(month);
    let d = i64::from(day);

    let adjusted_month = m.checked_sub(3)?;
    let adjusted_year = if adjusted_month < 0 {
        y.checked_sub(1)?
    } else {
        y
    };
    let adjusted_month = if adjusted_month < 0 {
        adjusted_month.checked_add(12)?
    } else {
        adjusted_month
    };

    let era = adjusted_year.div_euclid(400);
    let yoe = adjusted_year.checked_sub(era.checked_mul(400)?)?;
    let doy = (153_i64
        .checked_mul(adjusted_month.checked_add(2)?)?
        .checked_add(2)?)
    .div_euclid(5)
    .checked_add(d)?
    .checked_sub(1)?;
    let doe = yoe
        .checked_mul(365)?
        .checked_add(yoe.div_euclid(4))?
        .checked_sub(yoe.div_euclid(100))?
        .checked_add(doy)?;
    let days = era
        .checked_mul(146_097)?
        .checked_add(doe)?
        .checked_sub(719_468)?;

    u64::try_from(days).ok()
}
