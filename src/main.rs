use std::{path::PathBuf, time::Duration};

use app_settings::Settings;
use hotkeys::iced_key_to_livesplit_hotkey;
use iced::{Event, Size, Subscription, Task, Theme, event, keyboard, window};

use livesplit_core::{HotkeyConfig, hotkey::Hotkey};
use livesplit_state::LivesplitState;
use rfd::{AsyncMessageDialog, MessageDialogResult};
use state::splits_editor::{self, SplitsEditorState};
use ui::{edit_splits_window, main_window, settings_window};

mod app_settings;
mod hotkeys;
mod livesplit_state;
mod state;
mod ui;
mod widgets;

fn main() -> Result<(), iced::Error> {
    println!("Hello, world!");

    iced::daemon(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Clone, Debug)]
enum Message {
    // Empty
    None,

    // Window management
    WindowResized(window::Id, Size),
    WindowClosed(window::Id),
    OpenSettingsWindow,

    TimerTick,

    // Hotkeys
    KeyEvent(window::Id, keyboard::Event),
    HotkeyBoxChangedFocus(usize, bool),
    ClearHotkey(usize),
    SaveHotkeys,
    DiscardHotkeys,

    // File saving and loading
    TrySaveSplits,
    SaveSplits(PathBuf),
    TryLoadSplits,
    LoadSplits(PathBuf),
    TryLoadLayout,
    LoadLayout(PathBuf),
    CloseRequested(window::Id),

    // Splits Editing
    OpenEditSplitsWindow,
    SplitsEditorMessage(splits_editor::Message),

    // Error
    ErrorOccurred { title: String, error: String },
}

pub struct HotkeyBox {
    label: &'static str,
    pub value: Option<Hotkey>,
}

impl HotkeyBox {
    pub const fn new(label: &'static str) -> Self {
        Self { label, value: None }
    }
}

pub const fn load_hotkeys_from_hks(livesplit_state: &LivesplitState, hotkeys: &mut [HotkeyBox; 9]) {
    let config = livesplit_state.hks.config();

    hotkeys[0].value = config.split;
    hotkeys[1].value = config.reset;
    hotkeys[2].value = config.undo;
    hotkeys[3].value = config.skip;
    hotkeys[4].value = config.pause;
    hotkeys[5].value = config.undo_all_pauses;
    hotkeys[6].value = config.previous_comparison;
    hotkeys[7].value = config.next_comparison;
    hotkeys[8].value = config.toggle_timing_method;
}

pub fn save_hotkeys_to_hks(
    livesplit_state: &mut LivesplitState,
    hotkeys: &[HotkeyBox; 9],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = livesplit_state.hks.config();

    config.split = hotkeys[0].value;
    config.reset = hotkeys[1].value;
    config.undo = hotkeys[2].value;
    config.skip = hotkeys[3].value;
    config.pause = hotkeys[4].value;
    config.undo_all_pauses = hotkeys[5].value;
    config.previous_comparison = hotkeys[6].value;
    config.next_comparison = hotkeys[7].value;
    config.toggle_timing_method = hotkeys[8].value;

    // first clear the config so that we don't get false duplicate errors

    livesplit_state.hks.set_config(HotkeyConfig {
        split: None,
        reset: None,
        undo: None,
        skip: None,
        pause: None,
        undo_all_pauses: None,
        previous_comparison: None,
        next_comparison: None,
        toggle_timing_method: None,
    })?;

    livesplit_state.hks.set_config(config)?;
    Ok(())
}

struct App {
    main_window: window::Id,
    settings_window: Option<window::Id>,
    edit_splits_window: Option<window::Id>,

    settings: Settings,

    main_window_width: u32,
    main_window_height: u32,

    livesplit_state: LivesplitState,

    splits_editor_state: Option<SplitsEditorState>,

    hotkeys: [HotkeyBox; 9],
    hotkey_focused: Option<usize>,
}
enum WindowType {
    Main,
    Settings,
    EditSplits,
    Untracked,
}
impl App {
    pub fn new() -> (Self, Task<Message>) {
        let (main_window, window_open_task) = window::open(window::Settings {
            exit_on_close_request: false,
            ..Default::default()
        });

        let settings = Settings::load().unwrap_or_default();

        (
            Self {
                main_window,
                settings_window: None,
                edit_splits_window: None,
                main_window_width: 0,
                main_window_height: 0,
                livesplit_state: LivesplitState::with_settings(&settings),
                splits_editor_state: None,
                settings,

                hotkeys: [
                    "Start/Split",
                    "Reset",
                    "Undo",
                    "Skip",
                    "Pause",
                    "Undo All Pauses",
                    "Previous Comparison",
                    "Next Comparison",
                    "Toggle Timing Method",
                ]
                .map(HotkeyBox::new),
                hotkey_focused: None,
            },
            window_open_task.discard(),
        )
    }
    pub fn view(&self, window: window::Id) -> iced::Element<'_, Message, Theme> {
        match self.identify_window(window) {
            WindowType::Main => main_window::view(self),
            WindowType::Settings => settings_window::view(self),
            WindowType::EditSplits => edit_splits_window::view(self),
            WindowType::Untracked => panic!("Tried to view untracked window"),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => {}
            Message::TimerTick => self
                .livesplit_state
                .update(self.main_window_width, self.main_window_height),
            Message::WindowResized(id, size) => {
                if id == self.main_window {
                    self.main_window_width = size.width as u32;
                    self.main_window_height = size.height as u32;
                }
                self.livesplit_state
                    .update(self.main_window_width, self.main_window_height);
            }
            Message::OpenEditSplitsWindow => {
                self.livesplit_state.disable_hotkeys().ok();

                let (id, window_task) = window::open(window::Settings::default());

                self.edit_splits_window = Some(id);

                self.splits_editor_state = Some(SplitsEditorState::new(&mut self.livesplit_state));
                return window_task.discard();
            }
            Message::OpenSettingsWindow => {
                self.livesplit_state.disable_hotkeys().ok();

                let (id, window_task) = window::open(window::Settings::default());

                self.settings_window = Some(id);

                load_hotkeys_from_hks(&self.livesplit_state, &mut self.hotkeys);
                return window_task.discard();
            }
            Message::DiscardHotkeys => {
                load_hotkeys_from_hks(&self.livesplit_state, &mut self.hotkeys);
            }
            Message::SaveHotkeys => {
                if let Err(e) = save_hotkeys_to_hks(&mut self.livesplit_state, &self.hotkeys) {
                    return Task::done(Message::ErrorOccurred {
                        title: "Failed to update hotkeys".to_owned(),
                        error: e.to_string(),
                    });
                };
                self.livesplit_state
                    .save_hotkeys_to_settings(&mut self.settings);
            }
            Message::WindowClosed(window) => match self.identify_window(window) {
                WindowType::Main => {
                    // the window is already closed - we can't do anything about this
                    self.settings.save().ok();
                    return iced::exit();
                }
                WindowType::Settings => {
                    self.settings_window = None;
                    self.hotkey_focused = None;

                    if let Err(e) = self.livesplit_state.enable_hotkeys() {
                        return Task::done(Message::ErrorOccurred {
                            title: "Failed to re-enable hotkeys".to_owned(),
                            error: e.to_string(),
                        });
                    };
                }
                WindowType::EditSplits => {
                    self.edit_splits_window = None;

                    if let Some(splits_editor_state) = self.splits_editor_state.take() {
                        splits_editor_state.close_window(&mut self.livesplit_state);
                    }
                }
                WindowType::Untracked => panic!("Tried to close untracked window"),
            },
            Message::KeyEvent(id, evt) => {
                if let keyboard::Event::KeyPressed {
                    physical_key,
                    modifiers,
                    ..
                } = evt
                {
                    if self.settings_window == Some(id) {
                        self.hotkey_focused.inspect(|f| {
                            self.hotkeys[*f].value =
                                iced_key_to_livesplit_hotkey(physical_key, modifiers);
                        });
                    }
                }
            }
            Message::HotkeyBoxChangedFocus(id, focus) => {
                if focus {
                    self.hotkey_focused = Some(id);
                } else if self.hotkey_focused.is_some_and(|new_id| new_id == id) {
                    self.hotkey_focused = None;
                }
            }
            Message::ClearHotkey(id) => {
                self.hotkeys[id].value.take();
            }
            Message::TryLoadSplits => {
                if self.livesplit_state.is_timer_mid_run() {
                    return Task::none();
                }
                let (load_task, lth) = Task::future(Self::get_load_splits_path()).abortable();

                let save_if_dirty_task = self.save_if_dirty(lth);

                return save_if_dirty_task.chain(load_task);
            }
            Message::LoadSplits(path) => {
                if let Err(e) = self.livesplit_state.load_splits(&path) {
                    return Task::done(Message::ErrorOccurred {
                        title: "Failed to load splits".to_owned(),
                        error: e.to_string(),
                    });
                }
                self.settings.splits_path.replace(path);
            }
            Message::TrySaveSplits => {
                return Task::future(Self::get_save_splits_path(None));
            }
            Message::SaveSplits(path) => {
                println!("Saving splits!");

                if let Err(e) = self.livesplit_state.save_splits(&path) {
                    return Task::done(Message::ErrorOccurred {
                        title: "Failed to save splits".to_owned(),
                        error: e.to_string(),
                    });
                }
            }
            Message::TryLoadLayout => {
                return Task::future(async {
                    match rfd::AsyncFileDialog::new()
                        //.add_filter("LiveSplit Splits Files", &["*.lss"])
                        .set_title("Load Layout")
                        .pick_file()
                        .await
                    {
                        Some(path) => Message::LoadLayout(path.path().to_owned()),
                        None => Message::None,
                    }
                });
            }
            Message::LoadLayout(path) => {
                if let Err(e) = self.livesplit_state.load_layout(&path) {
                    return Task::done(Message::ErrorOccurred {
                        title: "Failed to load layout".to_owned(),
                        error: e.to_string(),
                    });
                }
                self.settings.layout_path.replace(path);
            }
            Message::CloseRequested(window) => {
                if let WindowType::Main = self.identify_window(window) {
                    let (close_window_task, ct) = window::close::<Message>(window).abortable();

                    let save_if_dirty_task = self.save_if_dirty(ct);

                    // disgusting hacky nonsense but i dont understand iced's async model enough to make it any better.
                    // ideally some kind of like combination of and_then and chain
                    // i feel like if i knew what a monad was i could make this cleaner
                    let (middle_task, ct) = save_if_dirty_task.chain(close_window_task).abortable();

                    let timer_running_task = if self.livesplit_state.is_timer_mid_run() {
                        Task::future(async move {
                                            if  MessageDialogResult::No == AsyncMessageDialog::new()
                                                .set_buttons(rfd::MessageButtons::YesNo)
                                                .set_title("Quit?")
                                                .set_description(
                                                    "The timer is currently running. Are you sure you want to quit?",
                                                )
                                                .show()
                                                .await
                                            {
                                                    ct.abort();
                                            }
                                        }).discard()
                    } else {
                        Task::none()
                    };

                    return timer_running_task.chain(middle_task);
                }

                panic!("Tried to close untracked window")
            }
            Message::SplitsEditorMessage(message) => self
                .splits_editor_state
                .as_mut()
                .expect("Recieved a splits editor messagge when the splits editor was closed")
                .update(message),
            Message::ErrorOccurred { title, error } => {
                return Task::future(
                    rfd::AsyncMessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title(title)
                        .set_description(error)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show(),
                )
                .discard();
            }
        }

        Task::none()
    }

    pub const fn theme(&self, _: window::Id) -> Theme {
        Theme::Dark
    }

    pub fn title(&self, window: window::Id) -> String {
        match self.identify_window(window) {
            WindowType::Main => "LiveSplit".into(),
            WindowType::Settings => "Settings | LiveSplit".into(),
            WindowType::EditSplits => "Edit Splits | LiveSplit".into(),
            WindowType::Untracked => panic!("Tried to get title of untracked window"),
        }
    }

    pub fn identify_window(&self, window: window::Id) -> WindowType {
        if window == self.main_window {
            WindowType::Main
        } else if self.settings_window == Some(window) {
            WindowType::Settings
        } else if self.edit_splits_window == Some(window) {
            WindowType::EditSplits
        } else {
            WindowType::Untracked
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_secs_f64(1. / 60.)).map(|_| Message::TimerTick),
            window::resize_events().map(|x| Message::WindowResized(x.0, x.1)),
            window::close_requests().map(Message::CloseRequested),
            window::close_events().map(Message::WindowClosed),
            event::listen_with(key_press_event_listener),
        ])
    }

    async fn get_load_splits_path() -> Message {
        match rfd::AsyncFileDialog::new()
            //.add_filter("LiveSplit Splits Files", &["*.lss"])
            .set_title("Load splits")
            .pick_file()
            .await
        {
            Some(path) => Message::LoadSplits(path.path().to_owned()),
            None => Message::None,
        }
    }

    async fn get_save_splits_path(ct: Option<iced::task::Handle>) -> Message {
        match rfd::AsyncFileDialog::new()
            //.add_filter("LiveSplit Splits Files", &["*.lss"])
            .set_title("Load splits")
            .save_file()
            .await
        {
            Some(path) => Message::SaveSplits(path.path().to_owned()),
            None => {
                if let Some(ct) = ct {
                    ct.abort();
                }
                Message::None
            }
        }
    }

    fn save_if_dirty(&self, ct: iced::task::Handle) -> Task<Message> {
        if self.livesplit_state.is_dirty() {
            Task::future(async {
                match AsyncMessageDialog::new()
                    .set_buttons(rfd::MessageButtons::YesNoCancel)
                    .set_title("Save splits?")
                    .set_description("Your splits have been modified. Would you like to save them?")
                    .show()
                    .await
                {
                    rfd::MessageDialogResult::Yes => {
                        Some(Self::get_save_splits_path(Some(ct)).await)
                    }
                    rfd::MessageDialogResult::No => None,
                    rfd::MessageDialogResult::Cancel => {
                        ct.abort();
                        None
                    }
                    _ => unreachable!(),
                }
            })
            .and_then(Task::done)
        } else {
            Task::none()
        }
    }
}

fn key_press_event_listener(
    event: iced::event::Event,
    _status: iced::event::Status,
    id: window::Id,
) -> Option<Message> {
    match event {
        Event::Keyboard(event) if matches!(event, keyboard::Event::KeyPressed { .. }) => {
            Some(Message::KeyEvent(id, event))
        }
        _ => None,
    }
}
