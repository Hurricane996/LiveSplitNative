use livesplit_core::{RunEditor, run::editor};

use crate::livesplit_state::LivesplitState;

pub struct SplitsEditorState {
    pub offset_buffer: String,
    pub editor: RunEditor,
    pub editor_state: livesplit_core::run::editor::State,
    pub segment_time_buffers: Vec<String>,
    pub split_time_buffers: Vec<String>,
    pub best_segment_time_buffers: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    UpdateGameName(String),
    UpdateCategoryName(String),
    UpdateNumAttempts(String),
    UpdateOffsetBuffer(String),
    OffsetTextboxBlur,
    SelectRow(usize),

    UpdateSegmentName(String),
    SplitTimeBlur(usize),
    SegmentTimeBlur(usize),
    BestSegmentTimeBlur(usize),
    UpdateSplitTimeBuffer(String, usize),
    UpdateSegmentTimeBuffer(String, usize),
    UpdateBestSegmentTimeBuffer(String, usize),

    InsertAboveClicked,
    InsertBelowClicked,
    RemoveSegmentClicked,
    MoveUpClicked,
    MoveDownClicked,
}

impl Message {
    pub fn into_app_message(self) -> crate::Message {
        crate::Message::SplitsEditorMessage(self)
    }
}
// Safety: This entire impl block is spaghetti of unsafe unwraps, which needs fixed. However, none of this spaghetti
// leaks into the public so until we fix it, just tread lightly when updating this.
impl SplitsEditorState {
    pub fn new(livesplit_state: &mut LivesplitState) -> Self {
        let run = {
            let timer = livesplit_state.timer.read().expect("Timer lock poisoned!");

            timer.run().clone()
        };
        let mut editor = RunEditor::new(run).unwrap();
        let editor_state = editor.state();

        let mut me = Self {
            offset_buffer: String::new(),
            editor,
            editor_state,
            segment_time_buffers: vec![],
            split_time_buffers: vec![],
            best_segment_time_buffers: vec![],
        };

        me.offset_buffer = me.editor.state().offset.clone();
        me.update_buffers();

        me
    }

    // SAFETY: this should only be called if editor_state is known to be Some

    fn update_buffers_for_row(&mut self, idx: usize, state: &editor::State) {
        self.split_time_buffers[idx] = state.segments[idx].split_time.clone();
        // this code is not panic safe unless documented contract is followed
        self.segment_time_buffers[idx] = state.segments[idx].segment_time.clone();
        // this code is not panic safe unless documented contract is followed
        self.best_segment_time_buffers[idx] = state.segments[idx].best_segment_time.clone();
    }

    // SAFETY: this should only be called if editor_state is known to be Some
    fn update_buffers(&mut self) {
        let state = self.editor.state();

        let num_rows = state.segments.len();

        self.split_time_buffers = vec![String::new(); num_rows];
        self.segment_time_buffers = vec![String::new(); num_rows];
        self.best_segment_time_buffers = vec![String::new(); num_rows];

        for idx in 0..num_rows {
            // Safety: if editor_state was none we would have thrown earlier in the function
            self.update_buffers_for_row(idx, &state);
        }
    }
    pub fn close_window(mut self, livesplit_state: &mut LivesplitState) {
        // todo don't auto apply

        self.editor.parse_and_set_offset(&self.offset_buffer).ok();

        let run = self.editor.close();

        let mut timer = livesplit_state.timer.write().expect("Timer lock poisoned!");
        timer.replace_run(run, false).ok();
    }

    pub fn update(&mut self, message: Message) {
        println!("{message:?}");
        match message {
            Message::UpdateGameName(new_game_name) => {
                self.editor.set_game_name(new_game_name);
            }
            Message::UpdateCategoryName(new_category_name) => {
                self.editor.set_category_name(new_category_name);
            }
            Message::UpdateNumAttempts(new_num_attempts) => {
                if let Ok(num_attempts) = new_num_attempts.parse::<u32>() {
                    self.editor.set_attempt_count(num_attempts);
                }
            }
            Message::UpdateOffsetBuffer(s) => self.offset_buffer = s,
            Message::OffsetTextboxBlur => {
                match self.editor.parse_and_set_offset(&self.offset_buffer) {
                    Ok(_) => {}
                    Err(_) => self.offset_buffer = self.editor.state().offset,
                };
            }
            Message::SelectRow(row) => {
                println!("row {row} selected");
                self.editor.select_only(row);
            }
            Message::UpdateSegmentName(new_name) => {
                self.editor.active_segment().set_name(new_name);
            }
            Message::SplitTimeBlur(_idx) => {
                self.update_buffers();
            }
            Message::SegmentTimeBlur(_idx) => {
                self.update_buffers();
            }
            Message::BestSegmentTimeBlur(idx) => {
                let state = self.editor.state();
                self.update_buffers_for_row(idx, &state);
            }
            Message::UpdateSplitTimeBuffer(text, idx) => {
                self.split_time_buffers[idx] = text.clone();
                self.editor
                    .active_segment()
                    .parse_and_set_split_time(&self.split_time_buffers[idx])
                    .ok();
            }
            Message::UpdateSegmentTimeBuffer(text, idx) => {
                self.segment_time_buffers[idx] = text;
                self.editor
                    .active_segment()
                    .parse_and_set_segment_time(&self.segment_time_buffers[idx])
                    .ok();
            }
            Message::UpdateBestSegmentTimeBuffer(text, idx) => {
                self.best_segment_time_buffers[idx] = text;
                self.editor
                    .active_segment()
                    .parse_and_set_best_segment_time(&self.best_segment_time_buffers[idx])
                    .ok();
            }
            Message::InsertAboveClicked => {
                self.editor.insert_segment_above();
                self.update_buffers();
            }
            Message::InsertBelowClicked => {
                self.editor.insert_segment_below();
                self.update_buffers();
            }
            Message::RemoveSegmentClicked => {
                self.editor.remove_segments();
                self.update_buffers();
            }
            Message::MoveUpClicked => {
                self.editor.move_segments_up();
                self.update_buffers();
            }
            Message::MoveDownClicked => {
                self.editor.move_segments_down();
                self.update_buffers();
            }
        }
        self.editor_state = self.editor.state();
    }
}
