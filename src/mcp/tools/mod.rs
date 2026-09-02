use std::fmt::Write;

use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData as McpError, schemars::JsonSchema, tool, tool_router};
use serde::Deserialize;

use crate::vault::{VaultReader, count_notes_recursive};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListNotesParams {
    /// Optional folder path relative to vault root (e.g. "Daily", "Projects")
    folder: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadNoteParams {
    /// Relative path to the note (e.g. "Daily/2025-01-15.md")
    path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchNotesParams {
    /// Search query to match against note content
    query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFoldersParams {
    /// Optional subfolder to list (defaults to vault root)
    folder: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteNoteParams {
    /// Relative path to the note (e.g. "Daily/2025-01-15.md")
    path: String,
    /// Full note content including optional YAML frontmatter
    content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteNoteParams {
    /// Relative path to the note (e.g. "Daily/2025-01-15.md")
    path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetNoteMetadataParams {
    /// Relative path to the note (e.g. "Daily/2025-01-15.md")
    path: String,
}

#[derive(Clone)]
pub struct AperioTools {
    vault: VaultReader,
}

impl AperioTools {
    pub const fn new(vault: VaultReader) -> Self {
        Self { vault }
    }
}

#[tool_router(server_handler)]
impl AperioTools {
    #[tool(description = "List all notes in the vault, optionally filtered by folder")]
    async fn list_notes(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<ListNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        let notes = self
            .vault
            .list_notes(params.folder.as_deref())
            .map_err(|e| McpError::internal_error(format!("Failed to list notes: {e}"), None))?;

        if notes.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text(
                "No notes found.".to_string(),
            )]));
        }

        let mut output = String::from("# Notes\n\n");
        for note in &notes {
            let tags_str = if note.tags.is_empty() {
                String::new()
            } else {
                format!(" `{}`", note.tags.join("`, `"))
            };
            let suffix = if tags_str.is_empty() {
                "`".to_string()
            } else {
                format!("`{tags_str}")
            };
            let _ = write!(
                output,
                "- **{}** — `{}{}",
                note.title,
                note.path.display(),
                suffix
            );
            output.push('\n');
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(output)]))
    }

    #[tool(description = "Read a note's content and frontmatter by relative path")]
    async fn read_note(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<ReadNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        let note = self
            .vault
            .read_note(&params.path)
            .map_err(|e| McpError::internal_error(format!("Failed to read note: {e}"), None))?;

        let mut output = String::new();

        if let Some(fm) = &note.frontmatter {
            output.push_str("---\n");
            if let Some(ref title) = fm.title {
                let _ = writeln!(output, "title: {title}");
            }
            if !fm.tags.is_empty() {
                let _ = writeln!(output, "tags: [{}]", fm.tags.join(", "));
            }
            output.push_str("---\n\n");
        }

        output.push_str(&note.content);

        Ok(CallToolResult::success(vec![ContentBlock::text(output)]))
    }

    #[tool(description = "Search notes by content query (full-text search)")]
    async fn search_notes(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<SearchNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        let query = params.query.to_lowercase();
        let notes = self
            .vault
            .list_notes(None)
            .map_err(|e| McpError::internal_error(format!("Failed to list notes: {e}"), None))?;

        let mut matches = Vec::new();
        for summary in &notes {
            if let Ok(note) = self.vault.read_note(&summary.path.to_string_lossy())
                && (note.content.to_lowercase().contains(&query)
                    || summary.title.to_lowercase().contains(&query)
                    || summary.tags.iter().any(|t| t.to_lowercase().contains(&query)))
            {
                matches.push(summary.clone());
            }
        }

        if matches.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "No notes matching '{query}'"
            ))]));
        }

        let mut output = format!("# Search results for '{query}'\n\n");
        for note in &matches {
            let _ = writeln!(output, "- **{}** — `{}`", note.title, note.path.display());
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(output)]))
    }

    #[tool(description = "List vault folders with note counts")]
    async fn list_folders(
        &self,
        rmcp::handler::server::wrapper::Parameters(_params): rmcp::handler::server::wrapper::Parameters<ListFoldersParams>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.vault.root().to_path_buf();

        let mut folders: Vec<(String, usize)> = Vec::new();
        count_notes_recursive(&root, "", &mut folders)
            .map_err(|e| McpError::internal_error(format!("Failed to list folders: {e}"), None))?;

        folders.sort_by(|a, b| a.0.cmp(&b.0));

        let mut output = String::from("# Vault Folders\n\n");
        for (name, count) in &folders {
            let display_name = if name.is_empty() { "/" } else { name };
            let _ = writeln!(output, "- **{display_name}** — {count} notes");
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(output)]))
    }

    #[tool(description = "Create or overwrite a note with given content. Content should include optional YAML frontmatter (---\\ntitle: ...\\ntags: [...]\\n---).")]
    async fn write_note(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<WriteNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        let note = self
            .vault
            .write_note(&params.path, &params.content)
            .map_err(|e| McpError::internal_error(format!("Failed to write note: {e}"), None))?;

        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "Note '{}' written to `{}`",
            note.title,
            note.path.display()
        ))]))
    }

    #[tool(description = "Soft-delete a note by moving it to .trash/ folder")]
    async fn delete_note(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<DeleteNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        let trash_path = self
            .vault
            .delete_note(&params.path)
            .map_err(|e| McpError::internal_error(format!("Failed to delete note: {e}"), None))?;

        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "Note moved to `{}`",
            trash_path.display()
        ))]))
    }

    #[tool(description = "Read frontmatter and tags without the note body")]
    async fn get_note_metadata(
        &self,
        rmcp::handler::server::wrapper::Parameters(params): rmcp::handler::server::wrapper::Parameters<GetNoteMetadataParams>,
    ) -> Result<CallToolResult, McpError> {
        let summary = self
            .vault
            .note_metadata(&params.path)
            .map_err(|e| McpError::internal_error(format!("Failed to read metadata: {e}"), None))?;

        let mut output = format!("# Metadata for `{}`\n\n", summary.path.display());
        let _ = writeln!(output, "- **Title:** {}", summary.title);
        if summary.tags.is_empty() {
            let _ = writeln!(output, "- **Tags:** (none)");
        } else {
            let _ = writeln!(output, "- **Tags:** `{}`", summary.tags.join("`, `"));
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(output)]))
    }
}
