use crate::models::{Clip, ExportFormat};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct Exporter;

impl Exporter {
    pub fn export_clips(clips: &[Clip], format: ExportFormat, output_path: Option<&Path>) -> Result<String> {
        let content = match format {
            ExportFormat::Json => Self::to_json(clips)?,
            ExportFormat::Csv => Self::to_csv(clips)?,
            ExportFormat::Html => Self::to_html(clips)?,
        };

        if let Some(path) = output_path {
            fs::write(path, &content)
                .with_context(|| format!("Failed to write export to path {:?}", path))?;
        }

        Ok(content)
    }

    pub fn to_json(clips: &[Clip]) -> Result<String> {
        serde_json::to_string_pretty(clips).context("Failed to serialize clips to JSON")
    }

    pub fn to_csv(clips: &[Clip]) -> Result<String> {
        let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());

        wtr.write_record([
            "id",
            "url",
            "title",
            "description",
            "tags",
            "notes",
            "author",
            "site_name",
            "reading_time_mins",
            "date_saved",
            "date_updated",
        ])?;

        for clip in clips {
            wtr.write_record([
                clip.id.to_string(),
                clip.url.clone(),
                clip.title.clone(),
                clip.description.clone().unwrap_or_default(),
                clip.tags.join(";"),
                clip.notes.clone().unwrap_or_default(),
                clip.author.clone().unwrap_or_default(),
                clip.site_name.clone().unwrap_or_default(),
                clip.reading_time_mins.map_or(String::new(), |r| r.to_string()),
                clip.date_saved.to_rfc3339(),
                clip.date_updated.to_rfc3339(),
            ])?;
        }

        let bytes = wtr.into_inner().context("Failed to generate CSV data")?;
        String::from_utf8(bytes).context("CSV output is not valid UTF-8")
    }

    pub fn to_html(clips: &[Clip]) -> Result<String> {
        let clips_json = serde_json::to_string(clips).unwrap_or_else(|_| "[]".to_string());

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Clipper - Saved Clips ({count})</title>
  <style>
    :root {{
      --bg-color: #0f172a;
      --card-bg: #1e293b;
      --card-border: #334155;
      --text-main: #f8fafc;
      --text-muted: #94a3b8;
      --accent: #38bdf8;
      --accent-hover: #0284c7;
      --tag-bg: #0f766e;
      --tag-text: #ccfbf1;
      --font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      background-color: var(--bg-color);
      color: var(--text-main);
      font-family: var(--font-family);
      line-height: 1.5;
      padding: 2rem;
    }}
    header {{
      max-width: 1200px;
      margin: 0 auto 2rem auto;
      display: flex;
      flex-wrap: wrap;
      justify-content: space-between;
      align-items: center;
      gap: 1rem;
      border-bottom: 1px solid var(--card-border);
      padding-bottom: 1.5rem;
    }}
    h1 {{
      font-size: 1.8rem;
      font-weight: 700;
      color: var(--accent);
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }}
    .search-box {{
      flex: 1;
      max-width: 400px;
      position: relative;
    }}
    .search-box input {{
      width: 100%;
      padding: 0.6rem 1rem;
      border-radius: 8px;
      border: 1px solid var(--card-border);
      background: var(--card-bg);
      color: var(--text-main);
      font-size: 0.95rem;
      outline: none;
    }}
    .search-box input:focus {{
      border-color: var(--accent);
    }}
    .grid {{
      max-width: 1200px;
      margin: 0 auto;
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
      gap: 1.5rem;
    }}
    .card {{
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 1.25rem;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      transition: transform 0.15s ease, border-color 0.15s ease;
    }}
    .card:hover {{
      transform: translateY(-2px);
      border-color: var(--accent);
    }}
    .card-title {{
      font-size: 1.1rem;
      font-weight: 600;
      margin-bottom: 0.5rem;
    }}
    .card-title a {{
      color: var(--text-main);
      text-decoration: none;
    }}
    .card-title a:hover {{
      color: var(--accent);
    }}
    .card-desc {{
      font-size: 0.9rem;
      color: var(--text-muted);
      margin-bottom: 1rem;
      display: -webkit-box;
      -webkit-line-clamp: 3;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }}
    .tags {{
      display: flex;
      flex-wrap: wrap;
      gap: 0.4rem;
      margin-bottom: 1rem;
    }}
    .tag {{
      background: var(--tag-bg);
      color: var(--tag-text);
      font-size: 0.75rem;
      font-weight: 500;
      padding: 0.2rem 0.6rem;
      border-radius: 9999px;
      cursor: pointer;
    }}
    .meta {{
      font-size: 0.8rem;
      color: var(--text-muted);
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-top: 1px solid var(--card-border);
      padding-top: 0.75rem;
      margin-top: auto;
    }}
    .site-info {{
      display: flex;
      align-items: center;
      gap: 0.4rem;
    }}
  </style>
</head>
<body>
  <header>
    <h1>🔖 Clipper Collection ({count})</h1>
    <div class="search-box">
      <input type="text" id="searchInput" placeholder="Search clips by title, tag, or content..." oninput="filterClips()">
    </div>
  </header>
  <div class="grid" id="clipsGrid"></div>

  <script>
    const clips = {clips_json};

    function renderClips(items) {{
      const container = document.getElementById('clipsGrid');
      if (items.length === 0) {{
        container.innerHTML = '<p style="grid-column: 1/-1; text-align: center; color: var(--text-muted);">No matching clips found.</p>';
        return;
      }}
      container.innerHTML = items.map(clip => `
        <div class="card">
          <div>
            <div class="card-title">
              <a href="${{clip.url}}" target="_blank" rel="noopener">${{escapeHtml(clip.title || clip.url)}}</a>
            </div>
            <p class="card-desc">${{escapeHtml(clip.description || clip.notes || 'No description provided.')}}</p>
            <div class="tags">
              ${{clip.tags.map(t => `<span class="tag" onclick="filterByTag('${{t}}')">#${{escapeHtml(t)}}</span>`).join('')}}
            </div>
          </div>
          <div class="meta">
            <span class="site-info">
              ${{clip.site_name ? escapeHtml(clip.site_name) : new URL(clip.url).hostname}}
            </span>
            <span>${{new Date(clip.date_saved).toLocaleDateString()}}</span>
          </div>
        </div>
      `).join('');
    }}

    function filterClips() {{
      const q = document.getElementById('searchInput').value.toLowerCase().trim();
      if (!q) {{
        renderClips(clips);
        return;
      }}
      const filtered = clips.filter(c => 
        (c.title && c.title.toLowerCase().includes(q)) ||
        (c.url && c.url.toLowerCase().includes(q)) ||
        (c.description && c.description.toLowerCase().includes(q)) ||
        (c.notes && c.notes.toLowerCase().includes(q)) ||
        (c.tags && c.tags.some(t => t.toLowerCase().includes(q)))
      );
      renderClips(filtered);
    }}

    function filterByTag(tag) {{
      document.getElementById('searchInput').value = tag;
      filterClips();
    }}

    function escapeHtml(str) {{
      return String(str)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
    }}

    renderClips(clips);
  </script>
</body>
</html>
"#,
            count = clips.len(),
            clips_json = clips_json
        );

        Ok(html)
    }
}
