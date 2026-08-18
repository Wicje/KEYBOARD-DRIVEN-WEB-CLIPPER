use crate::db::Database;
use crate::models::{Clip, ClipFilter, ExportFormat, TagCount};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Command,
    AddClip,
    EditTags,
    Export,
    ConfirmDelete,
    HelpModal,
}

#[derive(Debug, Clone)]
pub struct AddClipForm {
    pub url: String,
    pub tags: String,
    pub notes: String,
    pub active_field: usize, // 0: URL, 1: Tags, 2: Notes
}

impl Default for AddClipForm {
    fn default() -> Self {
        Self {
            url: String::new(),
            tags: String::new(),
            notes: String::new(),
            active_field: 0,
        }
    }
}

pub struct App<'a> {
    pub db: &'a Database,
    pub clips: Vec<Clip>,
    pub selected_index: usize,
    pub detail_scroll: u16,
    pub mode: InputMode,
    pub previous_mode: InputMode,
    pub search_query: String,
    pub command_input: String,
    pub tag_input: String,
    pub export_path_input: String,
    pub export_format: ExportFormat,
    pub add_form: AddClipForm,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub tags_summary: Vec<TagCount>,
}

impl<'a> App<'a> {
    pub fn new(db: &'a Database) -> Result<Self> {
        let mut app = Self {
            db,
            clips: Vec::new(),
            selected_index: 0,
            detail_scroll: 0,
            mode: InputMode::Normal,
            previous_mode: InputMode::Normal,
            search_query: String::new(),
            command_input: String::new(),
            tag_input: String::new(),
            export_path_input: String::from("clipper_export.json"),
            export_format: ExportFormat::Json,
            add_form: AddClipForm::default(),
            status_message: None,
            tags_summary: Vec::new(),
        };
        app.reload_clips()?;
        app.reload_tags()?;
        Ok(app)
    }

    pub fn reload_clips(&mut self) -> Result<()> {
        let filter = ClipFilter {
            query: if self.search_query.is_empty() {
                None
            } else {
                Some(self.search_query.clone())
            },
            ..Default::default()
        };

        self.clips = self.db.list_clips(&filter)?;
        if self.clips.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.clips.len() {
            self.selected_index = self.clips.len() - 1;
        }
        self.detail_scroll = 0;
        Ok(())
    }

    pub fn reload_tags(&mut self) -> Result<()> {
        self.tags_summary = self.db.list_tags()?;
        Ok(())
    }

    pub fn selected_clip(&self) -> Option<&Clip> {
        self.clips.get(self.selected_index)
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_message = Some((msg.into(), is_error));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn next_clip(&mut self) {
        if !self.clips.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.clips.len();
            self.detail_scroll = 0;
        }
    }

    pub fn previous_clip(&mut self) {
        if !self.clips.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.clips.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.detail_scroll = 0;
        }
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(2);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(2);
    }
}
