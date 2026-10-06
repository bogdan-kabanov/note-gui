use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::models::{Backlink, GraphData, GraphEdge, GraphNode, NoteSummary, SearchHit, TagInfo};
use crate::parse::parse_note;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS notes (
    path TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    word_count INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS tags (
    path TEXT NOT NULL,
    tag TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS aliases (
    path TEXT NOT NULL,
    alias TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS headings (
    path TEXT NOT NULL,
    level INTEGER NOT NULL,
    text TEXT NOT NULL,
    line INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS links (
    source_path TEXT NOT NULL,
    target_name TEXT NOT NULL,
    alias TEXT
);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_path);
";

pub struct NoteIndex {
    connection: Connection,
}

struct Resolver {
    paths: Vec<String>,
    titles: HashMap<String, String>,
    by_exact: HashMap<String, String>,
    stems: HashMap<String, Vec<String>>,
    aliases: HashMap<String, Vec<String>>,
}

impl NoteIndex {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let connection = Connection::open(path).map_err(|err| err.to_string())?;
        connection
            .execute_batch(SCHEMA)
            .map_err(|err| err.to_string())?;
        Ok(Self { connection })
    }

    pub fn upsert(&self, path: &str, body: &str, updated_at_ms: i64) -> Result<(), String> {
        let hash = content_hash(body);
        let existing: Option<String> = self
            .connection
            .query_row(
                "SELECT content_hash FROM notes WHERE path = ?1",
                [path],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| err.to_string())?;
        if existing.as_deref() == Some(hash.as_str()) {
            return Ok(());
        }
        let parsed = parse_note(path, body);
        let transaction = self
            .connection
            .unchecked_transaction()
            .map_err(|err| err.to_string())?;
        transaction
            .execute(
                "INSERT INTO notes (path, title, body, content_hash, updated_at_ms, word_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(path) DO UPDATE SET
                    title = excluded.title,
                    body = excluded.body,
                    content_hash = excluded.content_hash,
                    updated_at_ms = excluded.updated_at_ms,
                    word_count = excluded.word_count",
                params![
                    path,
                    parsed.title,
                    body,
                    hash,
                    updated_at_ms,
                    parsed.word_count
                ],
            )
            .map_err(|err| err.to_string())?;
        for sql in [
            "DELETE FROM tags WHERE path = ?1",
            "DELETE FROM aliases WHERE path = ?1",
            "DELETE FROM headings WHERE path = ?1",
            "DELETE FROM links WHERE source_path = ?1",
        ] {
            transaction
                .execute(sql, [path])
                .map_err(|err| err.to_string())?;
        }
        for tag in &parsed.tags {
            transaction
                .execute(
                    "INSERT INTO tags (path, tag) VALUES (?1, ?2)",
                    params![path, tag],
                )
                .map_err(|err| err.to_string())?;
        }
        for alias in &parsed.aliases {
            transaction
                .execute(
                    "INSERT INTO aliases (path, alias) VALUES (?1, ?2)",
                    params![path, alias],
                )
                .map_err(|err| err.to_string())?;
        }
        for heading in &parsed.headings {
            transaction
                .execute(
                    "INSERT INTO headings (path, level, text, line) VALUES (?1, ?2, ?3, ?4)",
                    params![path, heading.level, heading.text, heading.line],
                )
                .map_err(|err| err.to_string())?;
        }
        for link in &parsed.links {
            transaction
                .execute(
                    "INSERT INTO links (source_path, target_name, alias) VALUES (?1, ?2, ?3)",
                    params![path, link.target_name, link.alias],
                )
                .map_err(|err| err.to_string())?;
        }
        transaction.commit().map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn remove(&self, path: &str) -> Result<(), String> {
        let transaction = self
            .connection
            .unchecked_transaction()
            .map_err(|err| err.to_string())?;
        for sql in [
            "DELETE FROM notes WHERE path = ?1",
            "DELETE FROM tags WHERE path = ?1",
            "DELETE FROM aliases WHERE path = ?1",
            "DELETE FROM headings WHERE path = ?1",
            "DELETE FROM links WHERE source_path = ?1",
        ] {
            transaction
                .execute(sql, [path])
                .map_err(|err| err.to_string())?;
        }
        transaction.commit().map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn retain_paths(&self, seen: &HashSet<String>) -> Result<(), String> {
        let mut statement = self
            .connection
            .prepare("SELECT path FROM notes")
            .map_err(|err| err.to_string())?;
        let paths = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|err| err.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;
        drop(statement);
        for path in paths {
            if !seen.contains(&path) {
                self.remove(&path)?;
            }
        }
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchHit>, String> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let pattern = format!("%{}%", escape_like(trimmed));
        let mut statement = self
            .connection
            .prepare(
                "SELECT path, title, body FROM notes
                 WHERE title LIKE ?1 ESCAPE '\\' OR path LIKE ?1 ESCAPE '\\' OR body LIKE ?1 ESCAPE '\\'
                 ORDER BY title COLLATE NOCASE
                 LIMIT 80",
            )
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([&pattern], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|err| err.to_string())?;
        let mut hits = Vec::new();
        for row in rows {
            let (path, title, body) = row.map_err(|err| err.to_string())?;
            let (line, snippet) = snippet_for(&body, trimmed);
            hits.push(SearchHit {
                path,
                title,
                line,
                snippet,
            });
        }
        Ok(hits)
    }

    pub fn summaries(&self) -> Result<Vec<NoteSummary>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT path, title FROM notes ORDER BY title COLLATE NOCASE")
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(NoteSummary {
                    path: row.get(0)?,
                    title: row.get(1)?,
                })
            })
            .map_err(|err| err.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())
    }

    pub fn tags(&self) -> Result<Vec<TagInfo>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT tag, path FROM tags ORDER BY tag COLLATE NOCASE, path COLLATE NOCASE")
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|err| err.to_string())?;
        let mut grouped: Vec<TagInfo> = Vec::new();
        for row in rows {
            let (tag, path) = row.map_err(|err| err.to_string())?;
            if let Some(info) = grouped.iter_mut().find(|item| item.tag == tag) {
                info.paths.push(path);
                info.count = info.paths.len() as u32;
            } else {
                grouped.push(TagInfo {
                    tag,
                    count: 1,
                    paths: vec![path],
                });
            }
        }
        Ok(grouped)
    }

    pub fn resolve(&self, target_name: &str) -> Result<Option<String>, String> {
        self.resolver()?.resolve(target_name)
    }

    pub fn backlinks(&self, path: &str) -> Result<Vec<Backlink>, String> {
        let resolver = self.resolver()?;
        let mut statement = self
            .connection
            .prepare("SELECT source_path, target_name, alias FROM links")
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|err| err.to_string())?;
        let mut links = Vec::new();
        for row in rows {
            let (source_path, target_name, alias) = row.map_err(|err| err.to_string())?;
            if source_path == path {
                continue;
            }
            let resolved = match resolver.resolve(&target_name) {
                Ok(value) => value,
                Err(_) => None,
            };
            if resolved.as_deref() == Some(path) {
                let source_title = resolver
                    .titles
                    .get(&source_path)
                    .cloned()
                    .unwrap_or_else(|| source_path.clone());
                links.push(Backlink {
                    source_path,
                    source_title,
                    alias,
                });
            }
        }
        Ok(links)
    }

    pub fn graph(&self) -> Result<GraphData, String> {
        let resolver = self.resolver()?;
        let nodes = resolver
            .paths
            .iter()
            .map(|path| GraphNode {
                path: path.clone(),
                title: resolver
                    .titles
                    .get(path)
                    .cloned()
                    .unwrap_or_else(|| path.clone()),
            })
            .collect();
        let mut statement = self
            .connection
            .prepare("SELECT source_path, target_name FROM links")
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|err| err.to_string())?;
        let mut seen = HashSet::new();
        let mut edges = Vec::new();
        for row in rows {
            let (source_path, target_name) = row.map_err(|err| err.to_string())?;
            let Some(target_path) = resolver.resolve(&target_name).ok().flatten() else {
                continue;
            };
            if target_path == source_path {
                continue;
            }
            let key = format!("{source_path}->{target_path}");
            if seen.insert(key) {
                edges.push(GraphEdge {
                    source_path,
                    target_path,
                });
            }
        }
        Ok(GraphData { nodes, edges })
    }

    fn resolver(&self) -> Result<Resolver, String> {
        let mut statement = self
            .connection
            .prepare("SELECT path, title FROM notes")
            .map_err(|err| err.to_string())?;
        let note_rows = statement
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|err| err.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;
        drop(statement);
        let mut statement = self
            .connection
            .prepare("SELECT path, alias FROM aliases")
            .map_err(|err| err.to_string())?;
        let alias_rows = statement
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|err| err.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;
        let mut resolver = Resolver {
            paths: Vec::new(),
            titles: HashMap::new(),
            by_exact: HashMap::new(),
            stems: HashMap::new(),
            aliases: HashMap::new(),
        };
        for (path, title) in note_rows {
            resolver.by_exact.insert(normalize(&path), path.clone());
            let stem = Path::new(&path)
                .file_stem()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            resolver
                .stems
                .entry(normalize(&stem))
                .or_default()
                .push(path.clone());
            resolver.titles.insert(path.clone(), title);
            resolver.paths.push(path);
        }
        for (path, alias) in alias_rows {
            resolver
                .aliases
                .entry(normalize(&alias))
                .or_default()
                .push(path);
        }
        Ok(resolver)
    }
}

impl Resolver {
    fn resolve(&self, target_name: &str) -> Result<Option<String>, String> {
        let raw = target_name.trim();
        if raw.is_empty() {
            return Ok(None);
        }
        let key = normalize(raw);
        if let Some(path) = self.by_exact.get(&key) {
            return Ok(Some(path.clone()));
        }
        if let Some(paths) = self.aliases.get(&key) {
            return unique_path(paths);
        }
        if !key.contains('/') {
            if let Some(paths) = self.stems.get(&key) {
                return unique_path(paths);
            }
        }
        let mut matches = Vec::new();
        for path in &self.paths {
            let normalized = normalize(path);
            if normalized == key || normalized.ends_with(&format!("/{key}")) {
                matches.push(path.clone());
            }
        }
        unique_path(&matches)
    }
}

fn unique_path(paths: &[String]) -> Result<Option<String>, String> {
    if paths.is_empty() {
        return Ok(None);
    }
    if paths.len() == 1 {
        return Ok(Some(paths[0].clone()));
    }
    Err("Несколько заметок подходят под ссылку".into())
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(".md")
        .trim_end_matches(".MD")
        .replace('\\', "/")
        .to_lowercase()
}

fn content_hash(body: &str) -> String {
    let digest = Sha256::digest(body.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn escape_like(query: &str) -> String {
    query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn snippet_for(body: &str, query: &str) -> (u32, String) {
    let query_lower = query.to_lowercase();
    for (index, line) in body.lines().enumerate() {
        if line.to_lowercase().contains(&query_lower) {
            return (index as u32 + 1, trim_snippet(line));
        }
    }
    let fallback = body.lines().find(|line| !line.trim().is_empty()).unwrap_or("");
    (1, trim_snippet(fallback))
}

fn trim_snippet(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.chars().count() <= 180 {
        trimmed.to_string()
    } else {
        let short: String = trimmed.chars().take(180).collect();
        format!("{short}…")
    }
}
