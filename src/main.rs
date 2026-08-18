use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use clap::Parser;
use clipper::cli::{self, Cli, Commands, TagSubcommands};
use clipper::config::Config;
use clipper::db::Database;
use clipper::export::Exporter;
use clipper::fetcher::MetadataFetcher;
use clipper::models::{Clip, ClipFilter, NewClip, UpdateClip};
use clipper::tui;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    let db_path = cli.db.unwrap_or(config.db_path);
    let db = Database::open(&db_path)?;

    // If user passed --tui or no subcommand, launch TUI
    if cli.tui || cli.command.is_none() {
        return tui::run_tui(&db);
    }

    match cli.command.unwrap() {
        Commands::Clip(args) => handle_clip(&db, args),
        Commands::Search(args) => handle_search(&db, args),
        Commands::List(args) => handle_list(&db, args),
        Commands::Show(args) => handle_show(&db, args),
        Commands::Open(args) => handle_open(&db, args),
        Commands::Edit(args) => handle_edit(&db, args),
        Commands::Delete(args) => handle_delete(&db, args),
        Commands::Tag(args) => handle_tag(&db, args),
        Commands::Export(args) => handle_export(&db, args),
        Commands::Tui => tui::run_tui(&db),
    }
}

fn handle_clip(db: &Database, args: cli::ClipArgs) -> Result<()> {
    let user_tags: Vec<String> = args
        .tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    println!("⚡ Clipping URL: {}", args.url);

    let new_clip = if args.no_fetch {
        NewClip {
            url: args.url.clone(),
            title: args.title.unwrap_or_else(|| args.url.clone()),
            description: None,
            tags: user_tags,
            notes: args.notes,
            content_text: None,
            screenshot_path: None,
            favicon_url: None,
            author: None,
            site_name: None,
            reading_time_mins: None,
        }
    } else {
        println!("🔍 Fetching web page metadata...");
        let fetcher = MetadataFetcher::new()?;
        fetcher.fetch_and_extract(&args.url, args.title, user_tags, args.notes)?
    };

    let clip = db.insert_clip(new_clip)?;
    println!("\n✅ Successfully clipped web page!");
    println!("   ID:       {}", clip.id);
    println!("   Title:    {}", clip.title);
    println!("   URL:      {}", clip.url);
    println!("   Tags:     {}", clip.tags_string());
    if let Some(desc) = &clip.description {
        println!("   Summary:  {}", desc);
    }
    if let Some(time) = clip.reading_time_mins {
        println!("   Est. Read: ~{} min", time);
    }

    Ok(())
}

fn handle_search(db: &Database, args: cli::SearchArgs) -> Result<()> {
    if args.tui {
        return tui::run_tui(db);
    }

    let from_date = parse_date_opt(args.from.as_deref())?;
    let to_date = parse_date_opt(args.to.as_deref())?;

    let filter = ClipFilter {
        query: args.query.clone(),
        tag: args.tag,
        from_date,
        to_date,
        limit: args.limit,
        offset: None,
    };

    let clips = db.list_clips(&filter)?;
    print_clips_table(&clips, args.query.as_deref().unwrap_or("All Clips"));

    Ok(())
}

fn handle_list(db: &Database, args: cli::ListArgs) -> Result<()> {
    if args.tui {
        return tui::run_tui(db);
    }

    let filter = ClipFilter {
        query: None,
        tag: args.tag.clone(),
        from_date: None,
        to_date: None,
        limit: Some(args.limit),
        offset: None,
    };

    let clips = db.list_clips(&filter)?;
    let heading = args.tag.as_ref().map_or("Saved Clips".to_string(), |t| format!("Clips tagged '#{}'", t));
    print_clips_table(&clips, &heading);

    Ok(())
}

fn handle_show(db: &Database, args: cli::ShowArgs) -> Result<()> {
    let clip = db
        .get_clip_by_id(args.id)?
        .ok_or_else(|| anyhow::anyhow!("Clip with ID {} not found", args.id))?;

    println!("==================================================");
    println!("📌 ID:           {}", clip.id);
    println!("📌 Title:        {}", clip.title);
    println!("📌 URL:          {}", clip.url);
    println!("📌 Tags:         {}", clip.tags_string());
    println!("📌 Saved Date:   {}", clip.date_saved.format("%Y-%m-%d %H:%M:%S UTC"));
    if let Some(author) = &clip.author {
        println!("📌 Author:       {}", author);
    }
    if let Some(site) = &clip.site_name {
        println!("📌 Site:         {}", site);
    }
    if let Some(read_time) = clip.reading_time_mins {
        println!("📌 Est Read:     ~{} min", read_time);
    }
    if let Some(desc) = &clip.description {
        println!("\n📝 Description:\n{}", desc);
    }
    if let Some(notes) = &clip.notes {
        println!("\n💡 Personal Notes:\n{}", notes);
    }
    if let Some(text) = &clip.content_text {
        println!("\n📄 Extracted Content Preview:\n{}", text);
    }
    println!("==================================================");

    Ok(())
}

fn handle_open(db: &Database, args: cli::OpenArgs) -> Result<()> {
    let clip = db
        .get_clip_by_id(args.id)?
        .ok_or_else(|| anyhow::anyhow!("Clip with ID {} not found", args.id))?;

    println!("🌐 Opening URL in browser: {}", clip.url);
    open::that(&clip.url).with_context(|| format!("Failed to launch default browser for URL: {}", clip.url))?;
    Ok(())
}

fn handle_edit(db: &Database, args: cli::EditArgs) -> Result<()> {
    let existing = db
        .get_clip_by_id(args.id)?
        .ok_or_else(|| anyhow::anyhow!("Clip with ID {} not found", args.id))?;

    let mut tags = existing.tags.clone();

    if let Some(new_tags_str) = args.tags {
        tags = new_tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if let Some(add_str) = args.add_tag {
        for t in add_str.split(',') {
            let clean = t.trim().to_lowercase();
            if !clean.is_empty() && !tags.contains(&clean) {
                tags.push(clean);
            }
        }
    }

    if let Some(remove_str) = args.remove_tag {
        let remove_vec: Vec<String> = remove_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect();
        tags.retain(|t| !remove_vec.contains(t));
    }

    let update = UpdateClip {
        id: args.id,
        title: args.title,
        description: None,
        tags: Some(tags),
        notes: args.notes,
    };

    let updated = db.update_clip(update)?;
    println!("✅ Clip {} updated successfully!", updated.id);
    println!("   Title: {}", updated.title);
    println!("   Tags:  {}", updated.tags_string());

    Ok(())
}

fn handle_delete(db: &Database, args: cli::DeleteArgs) -> Result<()> {
    let clip = db
        .get_clip_by_id(args.id)?
        .ok_or_else(|| anyhow::anyhow!("Clip with ID {} not found", args.id))?;

    if !args.force {
        println!("⚠️  Are you sure you want to delete clip #{}: '{}'? [y/N]", clip.id, clip.title);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    db.delete_clip(args.id)?;
    println!("🗑️  Clip #{} deleted.", args.id);
    Ok(())
}

fn handle_tag(db: &Database, args: cli::TagArgs) -> Result<()> {
    match args.command {
        TagSubcommands::List => {
            let tags = db.list_tags()?;
            if tags.is_empty() {
                println!("No tags found.");
                return Ok(());
            }

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![Cell::new("Tag").fg(Color::Cyan), Cell::new("Clips Count").fg(Color::Yellow)]);

            for tag_cnt in tags {
                table.add_row(vec![
                    Cell::new(format!("#{}", tag_cnt.tag)),
                    Cell::new(tag_cnt.count.to_string()),
                ]);
            }

            println!("{}", table);
        }
        TagSubcommands::Add { id, tags } => {
            let new_tags: Vec<String> = tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let updated_tags = db.add_tags_to_clip(id, &new_tags)?;
            println!("✅ Added tags to clip #{}. Current tags: {}", id, updated_tags.join(", "));
        }
        TagSubcommands::Remove { id, tags } => {
            let remove_tags: Vec<String> = tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let updated_tags = db.remove_tags_from_clip(id, &remove_tags)?;
            println!("✅ Removed tags from clip #{}. Current tags: {}", id, updated_tags.join(", "));
        }
        TagSubcommands::Rename { old_tag, new_tag } => {
            let count = db.rename_tag(&old_tag, &new_tag)?;
            println!("✅ Renamed tag '#{}' to '#{}' across {} clips.", old_tag, new_tag, count);
        }
        TagSubcommands::Delete { tag } => {
            let count = db.delete_tag(&tag)?;
            println!("🗑️ Removed tag '#{}' from {} clips.", tag, count);
        }
    }

    Ok(())
}

fn handle_export(db: &Database, args: cli::ExportArgs) -> Result<()> {
    let filter = ClipFilter {
        query: args.query,
        tag: args.tag,
        from_date: None,
        to_date: None,
        limit: None,
        offset: None,
    };

    let clips = db.list_clips(&filter)?;
    let content = Exporter::export_clips(&clips, args.format, args.output.as_deref())?;

    if args.output.is_none() {
        println!("{}", content);
    } else {
        println!(
            "✅ Exported {} clips in {} format to {:?}",
            clips.len(),
            args.format,
            args.output.unwrap()
        );
    }

    Ok(())
}

fn print_clips_table(clips: &[Clip], title: &str) {
    if clips.is_empty() {
        println!("No clips found matching '{}'.", title);
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID").fg(Color::Yellow),
            Cell::new("Title").fg(Color::Green),
            Cell::new("URL").fg(Color::Cyan),
            Cell::new("Tags").fg(Color::Magenta),
            Cell::new("Saved Date").fg(Color::DarkGrey),
        ]);

    for clip in clips {
        let tags_str = clip
            .tags
            .iter()
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join(" ");

        table.add_row(vec![
            Cell::new(clip.id.to_string()),
            Cell::new(clip.display_title()),
            Cell::new(&clip.url),
            Cell::new(tags_str),
            Cell::new(clip.date_saved.format("%Y-%m-%d").to_string()),
        ]);
    }

    println!("\n📋 {} (Total: {}):\n{}", title, clips.len(), table);
}

fn parse_date_opt(s: Option<&str>) -> Result<Option<DateTime<Utc>>> {
    match s {
        Some(date_str) => {
            let nd = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .with_context(|| format!("Invalid date format '{}', expected YYYY-MM-DD", date_str))?;
            let dt = nd.and_hms_opt(0, 0, 0).unwrap().and_utc();
            Ok(Some(dt))
        }
        None => Ok(None),
    }
}
