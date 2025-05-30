use livesplit_core::RunEditor;

use crate::livesplit_state::LivesplitState;

#[derive(Default)]
pub struct SplitsEditorState {
    pub offset_buffer: String,
    editor: Option<RunEditor>,
    pub editor_state: Option<livesplit_core::run::editor::State>,

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
    pub fn to_app_message(self) -> crate::Message {
        crate::Message::SplitsEditorMessage(self)
    }
}

impl SplitsEditorState {
    pub fn open_window(&mut self, livesplit_state: &mut LivesplitState) {
        let run = {
            let timer = livesplit_state.timer.read().unwrap();

            timer.run().clone()
        };

        self.editor = Some(RunEditor::new(run).unwrap());

        self.update_state();

        self.offset_buffer = self.editor_state.as_mut().unwrap().offset.clone();

        self.update_buffers();
    }

    pub fn update_state(&mut self) {
        self.editor_state = Some(self.editor.as_mut().unwrap().state())
    }

    pub fn update_buffers_for_row(&mut self, idx: usize) {
        println!("Fixing buffers for {idx}");
        self.split_time_buffers[idx] = self.editor_state.as_mut().unwrap().segments[idx]
            .split_time
            .clone();
        self.segment_time_buffers[idx] = self.editor_state.as_mut().unwrap().segments[idx]
            .segment_time
            .clone();

        self.best_segment_time_buffers[idx] = self.editor_state.as_mut().unwrap().segments[idx]
            .best_segment_time
            .clone();
    }

    pub fn update_buffers(&mut self) {
        let num_rows = self.editor_state.as_ref().unwrap().segments.len();

        self.split_time_buffers = vec![String::new(); num_rows];
        self.segment_time_buffers = vec![String::new(); num_rows];
        self.best_segment_time_buffers = vec![String::new(); num_rows];

        for idx in 0..num_rows {
            self.update_buffers_for_row(idx);
        }
    }
    pub fn close_window(&mut self, livesplit_state: &mut LivesplitState) {
        // todo don't auto_apply

        self.editor
            .as_mut()
            .unwrap()
            .parse_and_set_offset(&self.offset_buffer)
            .ok();

        if let Some(run) = self.editor.take().map(RunEditor::close) {
            let mut timer = livesplit_state.timer.write().unwrap();
            timer.replace_run(run, false).ok();
        }
        self.editor = None;
        self.editor_state = None;
    }

    pub fn editor(&self) -> Option<&RunEditor> {
        self.editor.as_ref()
    }

    #[allow(unused)]
    pub fn editor_mut(&mut self) -> Option<&mut RunEditor> {
        self.editor.as_mut()
    }

    pub fn update(&mut self, message: Message) {
        println!("{message:?}");
        match message {
            Message::UpdateGameName(new_game_name) => {
                self.editor.as_mut().map(|x| x.set_game_name(new_game_name));

                self.update_state();
            }
            Message::UpdateCategoryName(new_category_name) => {
                self.editor
                    .as_mut()
                    .map(|x| x.set_category_name(new_category_name));
                self.update_state();
            }
            Message::UpdateNumAttempts(new_num_attempts) => {
                if let Some(x) = self.editor.as_mut() {
                    if let Ok(num_attempts) = new_num_attempts.parse::<u32>() {
                        x.set_attempt_count(num_attempts);
                    }
                };
                self.update_state();
            }
            Message::UpdateOffsetBuffer(s) => self.offset_buffer = s,
            Message::OffsetTextboxBlur => {
                match self
                    .editor
                    .as_mut()
                    .unwrap()
                    .parse_and_set_offset(&self.offset_buffer)
                {
                    Ok(_) => {}
                    Err(_) => self.offset_buffer = self.editor.as_mut().unwrap().state().offset,
                };

                self.update_state();
            }
            Message::SelectRow(row) => {
                println!("row {row} selected");
                self.editor.as_mut().unwrap().select_only(row);
                self.update_state();
            }
            Message::UpdateSegmentName(new_name) => {
                self.editor
                    .as_mut()
                    .unwrap()
                    .active_segment()
                    .set_name(new_name);
                self.update_state();
            }
            Message::SplitTimeBlur(_idx) => {
                self.update_state();
                self.update_buffers();
            }
            Message::SegmentTimeBlur(_idx) => {
                self.update_state();
                self.update_buffers();
            }
            Message::BestSegmentTimeBlur(idx) => {
                self.update_state();
                self.update_buffers_for_row(idx);
            }
            Message::UpdateSplitTimeBuffer(text, idx) => {
                self.split_time_buffers[idx] = text.clone();
                self.editor
                    .as_mut()
                    .unwrap()
                    .active_segment()
                    .parse_and_set_split_time(&self.split_time_buffers[idx])
                    .ok();
            }
            Message::UpdateSegmentTimeBuffer(text, idx) => {
                self.segment_time_buffers[idx] = text;
                self.editor
                    .as_mut()
                    .unwrap()
                    .active_segment()
                    .parse_and_set_segment_time(&self.segment_time_buffers[idx])
                    .ok();
            }
            Message::UpdateBestSegmentTimeBuffer(text, idx) => {
                self.best_segment_time_buffers[idx] = text;
                self.editor
                    .as_mut()
                    .unwrap()
                    .active_segment()
                    .parse_and_set_best_segment_time(&self.best_segment_time_buffers[idx])
                    .ok();
            }
            Message::InsertAboveClicked => {
                self.editor.as_mut().unwrap().insert_segment_above();
                self.update_state();
                self.update_buffers();
            }
            Message::InsertBelowClicked => {
                self.editor.as_mut().unwrap().insert_segment_below();
                self.update_state();
                self.update_buffers();
            }
            Message::RemoveSegmentClicked => {
                self.editor.as_mut().unwrap().remove_segments();
                self.update_state();
                self.update_buffers();
            }
            Message::MoveUpClicked => {
                self.editor.as_mut().unwrap().move_segments_up();
                self.update_state();
                self.update_buffers();
            }
            Message::MoveDownClicked => {
                self.editor.as_mut().unwrap().move_segments_down();
                self.update_state();
                self.update_buffers();
            }
        }
    }
}
