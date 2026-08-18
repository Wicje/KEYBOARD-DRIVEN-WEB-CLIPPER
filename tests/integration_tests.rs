use clipper::config::Config;
use clipper::db::Database;
use clipper::export::Exporter;
use clipper::models::{ClipFilter, ExportFormat, NewClip, UpdateClip};
use tempfile::TempDir;

#[test]
fn test_xdg_config_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config::custom(temp_dir.path().to_path_buf()).unwrap();

    assert!(config.data_dir.exists());
    assert!(config.assets_dir.exists());
    assert!(config.config_dir.exists());
    assert_eq!(config.db_path, config.data_dir.join("clips.db"));
}

#[test]
fn test_database_crud_and_fts() {
    let db = Database::in_memory().unwrap();

    // Insert clip 1
    let clip1 = db
        .insert_clip(NewClip {
            url: "https://rust-lang.org".to_string(),
            title: "Rust Programming Language".to_string(),
            description: Some("Empowering everyone to build reliable and efficient software.".to_string()),
            tags: vec!["rust".to_string(), "programming".to_string(), "systems".to_string()],
            notes: Some("Essential reference site for Rust programming language.".to_string()),
            content_text: Some("Rust is a multi-paradigm, general-purpose programming language designed for performance and safety.".to_string()),
            screenshot_path: None,
            favicon_url: Some("https://rust-lang.org/favicon.ico".to_string()),
            author: Some("Rust Core Team".to_string()),
            site_name: Some("Rust Official".to_string()),
            reading_time_mins: Some(3),
        })
        .unwrap();

    assert_eq!(clip1.id, 1);
    assert_eq!(clip1.title, "Rust Programming Language");
    assert_eq!(clip1.tags, vec!["rust", "programming", "systems"]);

    // Insert clip 2
    let clip2 = db
        .insert_clip(NewClip {
            url: "https://news.ycombinator.com".to_string(),
            title: "Hacker News".to_string(),
            description: Some("Social news website focusing on computer science and entrepreneurship.".to_string()),
            tags: vec!["news".to_string(), "tech".to_string()],
            notes: Some("Check daily tech news.".to_string()),
            content_text: Some("Hacker News is a social news website run by Y Combinator.".to_string()),
            screenshot_path: None,
            favicon_url: None,
            author: None,
            site_name: Some("Y Combinator".to_string()),
            reading_time_mins: Some(1),
        })
        .unwrap();

    assert_eq!(clip2.id, 2);

    // Get by URL
    let fetched_url = db.get_clip_by_url("https://rust-lang.org").unwrap();
    assert!(fetched_url.is_some());
    assert_eq!(fetched_url.unwrap().id, 1);

    // List all
    let all_clips = db.list_clips(&ClipFilter::default()).unwrap();
    assert_eq!(all_clips.len(), 2);

    // Tag filter
    let rust_clips = db
        .list_clips(&ClipFilter {
            tag: Some("rust".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(rust_clips.len(), 1);
    assert_eq!(rust_clips[0].id, 1);

    // Full-Text Search for keyword 'multi-paradigm'
    let search_res = db
        .search_clips(&ClipFilter {
            query: Some("multi-paradigm".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(search_res.len(), 1);
    assert_eq!(search_res[0].id, 1);

    // Update clip
    let updated = db
        .update_clip(UpdateClip {
            id: 1,
            title: Some("Rust Language (Official)".to_string()),
            description: None,
            tags: Some(vec!["rust".to_string(), "cli".to_string()]),
            notes: Some("Updated notes".to_string()),
        })
        .unwrap();

    assert_eq!(updated.title, "Rust Language (Official)");
    assert_eq!(updated.tags, vec!["rust", "cli"]);
    assert_eq!(updated.notes.unwrap(), "Updated notes");

    // Delete clip 2
    let deleted = db.delete_clip(2).unwrap();
    assert!(deleted);
    assert_eq!(db.count_clips().unwrap(), 1);
}

#[test]
fn test_tag_operations() {
    let db = Database::in_memory().unwrap();

    db.insert_clip(NewClip {
        url: "https://example.com/1".to_string(),
        title: "Page 1".to_string(),
        description: None,
        tags: vec!["design".to_string(), "web".to_string()],
        notes: None,
        content_text: None,
        screenshot_path: None,
        favicon_url: None,
        author: None,
        site_name: None,
        reading_time_mins: None,
    })
    .unwrap();

    db.insert_clip(NewClip {
        url: "https://example.com/2".to_string(),
        title: "Page 2".to_string(),
        description: None,
        tags: vec!["design".to_string(), "inspiration".to_string()],
        notes: None,
        content_text: None,
        screenshot_path: None,
        favicon_url: None,
        author: None,
        site_name: None,
        reading_time_mins: None,
    })
    .unwrap();

    let tag_counts = db.list_tags().unwrap();
    assert_eq!(tag_counts.len(), 3);
    assert_eq!(tag_counts[0].tag, "design");
    assert_eq!(tag_counts[0].count, 2);

    // Add tag to clip 1
    let clip1_tags = db.add_tags_to_clip(1, &["css".to_string()]).unwrap();
    assert!(clip1_tags.contains(&"css".to_string()));

    // Rename 'design' -> 'ui'
    let renamed = db.rename_tag("design", "ui").unwrap();
    assert_eq!(renamed, 2);

    let ui_clips = db
        .list_clips(&ClipFilter {
            tag: Some("ui".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(ui_clips.len(), 2);

    // Delete tag 'web'
    let deleted_tag_cnt = db.delete_tag("web").unwrap();
    assert_eq!(deleted_tag_cnt, 1);
}

#[test]
fn test_export_formats() {
    let db = Database::in_memory().unwrap();

    db.insert_clip(NewClip {
        url: "https://example.com".to_string(),
        title: "Test Example".to_string(),
        description: Some("Test Description".to_string()),
        tags: vec!["test".to_string(), "demo".to_string()],
        notes: Some("Test note".to_string()),
        content_text: Some("Sample article content".to_string()),
        screenshot_path: None,
        favicon_url: Some("https://example.com/favicon.ico".to_string()),
        author: Some("Tester".to_string()),
        site_name: Some("Example Domain".to_string()),
        reading_time_mins: Some(2),
    })
    .unwrap();

    let clips = db.list_clips(&ClipFilter::default()).unwrap();

    // Export JSON
    let json_output = Exporter::export_clips(&clips, ExportFormat::Json, None).unwrap();
    assert!(json_output.contains("Test Example"));
    assert!(json_output.contains("https://example.com"));

    // Export CSV
    let csv_output = Exporter::export_clips(&clips, ExportFormat::Csv, None).unwrap();
    assert!(csv_output.contains("id,url,title"));
    assert!(csv_output.contains("Test Example"));

    // Export HTML
    let html_output = Exporter::export_clips(&clips, ExportFormat::Html, None).unwrap();
    assert!(html_output.contains("<!DOCTYPE html>"));
    assert!(html_output.contains("Clipper Collection"));
    assert!(html_output.contains("Test Example"));
}
