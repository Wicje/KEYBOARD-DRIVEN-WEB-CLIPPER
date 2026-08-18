use crate::models::{Clip, ClipFilter, NewClip, TagCount, UpdateClip};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open SQLite database at {:?}", db_path))?;

        // Enable foreign key support & WAL mode for performance
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )?;

        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY,
                version INTEGER NOT NULL UNIQUE,
                applied_at TEXT NOT NULL
            );",
        )?;

        let current_version: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version < 1 {
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS clips (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    url TEXT NOT NULL UNIQUE,
                    title TEXT NOT NULL,
                    description TEXT,
                    tags TEXT NOT NULL,
                    notes TEXT,
                    content_text TEXT,
                    screenshot_path TEXT,
                    favicon_url TEXT,
                    author TEXT,
                    site_name TEXT,
                    reading_time_mins INTEGER,
                    date_saved TEXT NOT NULL,
                    date_updated TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS clip_tags (
                    clip_id INTEGER NOT NULL,
                    tag TEXT NOT NULL,
                    PRIMARY KEY (clip_id, tag),
                    FOREIGN KEY (clip_id) REFERENCES clips(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_clip_tags_tag ON clip_tags(tag);

                CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
                    clip_id UNINDEXED,
                    title,
                    description,
                    content_text,
                    tags,
                    notes,
                    url
                );

                CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
                    INSERT INTO clips_fts(clip_id, title, description, content_text, tags, notes, url)
                    VALUES (new.id, new.title, COALESCE(new.description, ''), COALESCE(new.content_text, ''), new.tags, COALESCE(new.notes, ''), new.url);
                END;

                CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
                    DELETE FROM clips_fts WHERE clip_id = old.id;
                END;

                CREATE TRIGGER IF NOT EXISTS clips_au AFTER UPDATE ON clips BEGIN
                    DELETE FROM clips_fts WHERE clip_id = old.id;
                    INSERT INTO clips_fts(clip_id, title, description, content_text, tags, notes, url)
                    VALUES (new.id, new.title, COALESCE(new.description, ''), COALESCE(new.content_text, ''), new.tags, COALESCE(new.notes, ''), new.url);
                END;
                ",
            )?;

            let now = Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO migrations (version, applied_at) VALUES (1, ?1)",
                params![now],
            )?;
        }

        Ok(())
    }

    pub fn insert_clip(&self, new_clip: NewClip) -> Result<Clip> {
        let now = Utc::now().to_rfc3339();
        let tags_clean: Vec<String> = new_clip
            .tags
            .into_iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        let tags_str = tags_clean.join(",");

        self.conn.execute(
            "INSERT INTO clips (
                url, title, description, tags, notes, content_text,
                screenshot_path, favicon_url, author, site_name,
                reading_time_mins, date_saved, date_updated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                new_clip.url,
                new_clip.title,
                new_clip.description,
                tags_str,
                new_clip.notes,
                new_clip.content_text,
                new_clip.screenshot_path,
                new_clip.favicon_url,
                new_clip.author,
                new_clip.site_name,
                new_clip.reading_time_mins,
                now,
                now,
            ],
        )?;

        let clip_id = self.conn.last_insert_rowid();

        for tag in &tags_clean {
            self.conn.execute(
                "INSERT OR IGNORE INTO clip_tags (clip_id, tag) VALUES (?1, ?2)",
                params![clip_id, tag],
            )?;
        }

        self.get_clip_by_id(clip_id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve newly inserted clip"))
    }

    pub fn get_clip_by_id(&self, id: i64) -> Result<Option<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, description, tags, notes, content_text, screenshot_path, favicon_url, author, site_name, reading_time_mins, date_saved, date_updated
             FROM clips WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_clip(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_clip_by_url(&self, url: &str) -> Result<Option<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, description, tags, notes, content_text, screenshot_path, favicon_url, author, site_name, reading_time_mins, date_saved, date_updated
             FROM clips WHERE url = ?1",
        )?;

        let mut rows = stmt.query(params![url])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_clip(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn update_clip(&self, update: UpdateClip) -> Result<Clip> {
        let existing = self
            .get_clip_by_id(update.id)?
            .ok_or_else(|| anyhow::anyhow!("Clip with ID {} not found", update.id))?;

        let now = Utc::now().to_rfc3339();
        let title = update.title.unwrap_or(existing.title);
        let description = update.description.or(existing.description);
        let notes = update.notes.or(existing.notes);

        let tags = if let Some(t) = update.tags {
            t.into_iter()
                .map(|tag| tag.trim().to_lowercase())
                .filter(|tag| !tag.is_empty())
                .collect()
        } else {
            existing.tags
        };
        let tags_str = tags.join(",");

        self.conn.execute(
            "UPDATE clips SET title = ?1, description = ?2, tags = ?3, notes = ?4, date_updated = ?5 WHERE id = ?6",
            params![title, description, tags_str, notes, now, update.id],
        )?;

        // Update tag bridge table
        self.conn.execute("DELETE FROM clip_tags WHERE clip_id = ?1", params![update.id])?;
        for tag in &tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO clip_tags (clip_id, tag) VALUES (?1, ?2)",
                params![update.id, tag],
            )?;
        }

        self.get_clip_by_id(update.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated clip"))
    }

    pub fn delete_clip(&self, id: i64) -> Result<bool> {
        let rows_affected = self.conn.execute("DELETE FROM clips WHERE id = ?1", params![id])?;
        Ok(rows_affected > 0)
    }

    pub fn list_clips(&self, filter: &ClipFilter) -> Result<Vec<Clip>> {
        if let Some(query) = &filter.query {
            if !query.trim().is_empty() {
                return self.search_clips(filter);
            }
        }

        let mut sql = String::from(
            "SELECT c.id, c.url, c.title, c.description, c.tags, c.notes, c.content_text, c.screenshot_path, c.favicon_url, c.author, c.site_name, c.reading_time_mins, c.date_saved, c.date_updated
             FROM clips c ",
        );

        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(tag) = &filter.tag {
            sql.push_str("JOIN clip_tags ct ON c.id = ct.clip_id ");
            conditions.push("ct.tag = ?".to_string());
            params_vec.push(Box::new(tag.to_lowercase()));
        }

        if let Some(from) = &filter.from_date {
            conditions.push("c.date_saved >= ?".to_string());
            params_vec.push(Box::new(from.to_rfc3339()));
        }

        if let Some(to) = &filter.to_date {
            conditions.push("c.date_saved <= ?".to_string());
            params_vec.push(Box::new(to.to_rfc3339()));
        }

        if !conditions.is_empty() {
            sql.push_str("WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY c.date_saved DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = filter.offset {
                sql.push_str(&format!(" OFFSET {}", offset));
            }
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| Self::row_to_clip(row))?;

        let mut clips = Vec::new();
        for clip_res in rows {
            clips.push(clip_res?);
        }
        Ok(clips)
    }

    pub fn search_clips(&self, filter: &ClipFilter) -> Result<Vec<Clip>> {
        let query_str = filter.query.as_deref().unwrap_or("").trim();
        if query_str.is_empty() {
            return self.list_clips(&ClipFilter {
                query: None,
                ..filter.clone()
            });
        }

        // Try FTS search first, fallback to LIKE search if FTS syntax errors
        let fts_query = format!("*{}*", query_str.replace('"', "\"\""));

        let mut sql = String::from(
            "SELECT c.id, c.url, c.title, c.description, c.tags, c.notes, c.content_text, c.screenshot_path, c.favicon_url, c.author, c.site_name, c.reading_time_mins, c.date_saved, c.date_updated
             FROM clips c
             JOIN clips_fts fts ON c.id = fts.clip_id ",
        );

        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        conditions.push("clips_fts MATCH ?".to_string());
        params_vec.push(Box::new(fts_query.clone()));

        if let Some(tag) = &filter.tag {
            sql.push_str("JOIN clip_tags ct ON c.id = ct.clip_id ");
            conditions.push("ct.tag = ?".to_string());
            params_vec.push(Box::new(tag.to_lowercase()));
        }

        if let Some(from) = &filter.from_date {
            conditions.push("c.date_saved >= ?".to_string());
            params_vec.push(Box::new(from.to_rfc3339()));
        }

        if let Some(to) = &filter.to_date {
            conditions.push("c.date_saved <= ?".to_string());
            params_vec.push(Box::new(to.to_rfc3339()));
        }

        sql.push_str("WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(" ORDER BY rank, c.date_saved DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = match self.conn.prepare(&sql) {
            Ok(stmt) => stmt,
            Err(_) => {
                // Fallback to LIKE query if FTS query expression fails
                return self.search_clips_fallback(filter);
            }
        };

        let rows_res = stmt.query_map(params_refs.as_slice(), |row| Self::row_to_clip(row));
        match rows_res {
            Ok(rows) => {
                let mut clips = Vec::new();
                for clip_res in rows {
                    if let Ok(c) = clip_res {
                        clips.push(c);
                    }
                }
                if !clips.is_empty() {
                    return Ok(clips);
                }
                // If FTS returned 0 results, retry fallback search for sub-string matches
                self.search_clips_fallback(filter)
            }
            Err(_) => self.search_clips_fallback(filter),
        }
    }

    fn search_clips_fallback(&self, filter: &ClipFilter) -> Result<Vec<Clip>> {
        let query_pattern = format!("%{}%", filter.query.as_deref().unwrap_or(""));

        let mut sql = String::from(
            "SELECT c.id, c.url, c.title, c.description, c.tags, c.notes, c.content_text, c.screenshot_path, c.favicon_url, c.author, c.site_name, c.reading_time_mins, c.date_saved, c.date_updated
             FROM clips c ",
        );

        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(tag) = &filter.tag {
            sql.push_str("JOIN clip_tags ct ON c.id = ct.clip_id ");
            conditions.push("ct.tag = ?".to_string());
            params_vec.push(Box::new(tag.to_lowercase()));
        }

        conditions.push("(c.title LIKE ? OR c.description LIKE ? OR c.content_text LIKE ? OR c.notes LIKE ? OR c.tags LIKE ? OR c.url LIKE ?)".to_string());
        for _ in 0..6 {
            params_vec.push(Box::new(query_pattern.clone()));
        }

        if let Some(from) = &filter.from_date {
            conditions.push("c.date_saved >= ?".to_string());
            params_vec.push(Box::new(from.to_rfc3339()));
        }

        if let Some(to) = &filter.to_date {
            conditions.push("c.date_saved <= ?".to_string());
            params_vec.push(Box::new(to.to_rfc3339()));
        }

        sql.push_str("WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(" ORDER BY c.date_saved DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| Self::row_to_clip(row))?;

        let mut clips = Vec::new();
        for clip_res in rows {
            clips.push(clip_res?);
        }
        Ok(clips)
    }

    pub fn list_tags(&self) -> Result<Vec<TagCount>> {
        let mut stmt = self.conn.prepare(
            "SELECT tag, COUNT(*) as count FROM clip_tags GROUP BY tag ORDER BY count DESC, tag ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TagCount {
                tag: row.get(0)?,
                count: row.get::<_, i64>(1)? as usize,
            })
        })?;

        let mut tags = Vec::new();
        for r in rows {
            tags.push(r?);
        }
        Ok(tags)
    }

    pub fn add_tags_to_clip(&self, id: i64, new_tags: &[String]) -> Result<Vec<String>> {
        let clip = self
            .get_clip_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Clip with ID {} not found", id))?;

        let mut tags = clip.tags;
        for tag in new_tags {
            let clean = tag.trim().to_lowercase();
            if !clean.is_empty() && !tags.contains(&clean) {
                tags.push(clean);
            }
        }

        let updated = self.update_clip(UpdateClip {
            id,
            title: None,
            description: None,
            tags: Some(tags),
            notes: None,
        })?;

        Ok(updated.tags)
    }

    pub fn remove_tags_from_clip(&self, id: i64, tags_to_remove: &[String]) -> Result<Vec<String>> {
        let clip = self
            .get_clip_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Clip with ID {} not found", id))?;

        let remove_set: std::collections::HashSet<String> = tags_to_remove
            .iter()
            .map(|t| t.trim().to_lowercase())
            .collect();

        let tags: Vec<String> = clip
            .tags
            .into_iter()
            .filter(|t| !remove_set.contains(t))
            .collect();

        let updated = self.update_clip(UpdateClip {
            id,
            title: None,
            description: None,
            tags: Some(tags),
            notes: None,
        })?;

        Ok(updated.tags)
    }

    pub fn rename_tag(&self, old_tag: &str, new_tag: &str) -> Result<usize> {
        let old_clean = old_tag.trim().to_lowercase();
        let new_clean = new_tag.trim().to_lowercase();

        if old_clean == new_clean || old_clean.is_empty() || new_clean.is_empty() {
            return Ok(0);
        }

        // Get clips with old tag
        let mut stmt = self
            .conn
            .prepare("SELECT clip_id FROM clip_tags WHERE tag = ?1")?;
        let clip_ids: Vec<i64> = stmt
            .query_map(params![old_clean], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let count = clip_ids.len();
        for clip_id in clip_ids {
            if let Some(clip) = self.get_clip_by_id(clip_id)? {
                let mut tags = clip.tags;
                if let Some(idx) = tags.iter().position(|t| t == &old_clean) {
                    tags.remove(idx);
                }
                if !tags.contains(&new_clean) {
                    tags.push(new_clean.clone());
                }
                self.update_clip(UpdateClip {
                    id: clip_id,
                    title: None,
                    description: None,
                    tags: Some(tags),
                    notes: None,
                })?;
            }
        }

        Ok(count)
    }

    pub fn delete_tag(&self, tag: &str) -> Result<usize> {
        let tag_clean = tag.trim().to_lowercase();
        let mut stmt = self
            .conn
            .prepare("SELECT clip_id FROM clip_tags WHERE tag = ?1")?;
        let clip_ids: Vec<i64> = stmt
            .query_map(params![tag_clean], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let count = clip_ids.len();
        for clip_id in clip_ids {
            if let Some(clip) = self.get_clip_by_id(clip_id)? {
                let tags: Vec<String> = clip
                    .tags
                    .into_iter()
                    .filter(|t| t != &tag_clean)
                    .collect();
                self.update_clip(UpdateClip {
                    id: clip_id,
                    title: None,
                    description: None,
                    tags: Some(tags),
                    notes: None,
                })?;
            }
        }

        Ok(count)
    }

    pub fn count_clips(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    fn row_to_clip(row: &rusqlite::Row) -> Result<Clip, rusqlite::Error> {
        let tags_str: String = row.get("tags")?;
        let tags: Vec<String> = if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split(',').map(|s| s.trim().to_string()).collect()
        };

        let date_saved_str: String = row.get("date_saved")?;
        let date_updated_str: String = row.get("date_updated")?;

        let date_saved = DateTime::parse_from_rfc3339(&date_saved_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let date_updated = DateTime::parse_from_rfc3339(&date_updated_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Clip {
            id: row.get("id")?,
            url: row.get("url")?,
            title: row.get("title")?,
            description: row.get("description")?,
            tags,
            notes: row.get("notes")?,
            content_text: row.get("content_text")?,
            screenshot_path: row.get("screenshot_path")?,
            favicon_url: row.get("favicon_url")?,
            author: row.get("author")?,
            site_name: row.get("site_name")?,
            reading_time_mins: row.get("reading_time_mins")?,
            date_saved,
            date_updated,
        })
    }
}
