use std::{path::PathBuf, time::Duration};

use app_settings::Settings;
use hotkeys::iced_key_to_livesplit_hotkey;
use iced::{Event, Size, Subscription, Task, Theme, event, keyboard, window};

use livesplit_state::LivesplitState;
use rfd::{AsyncMessageDialog, MessageDialogResult};
use settings_window::HotkeyBox;

mod app_settings;
mod hotkeys;
mod livesplit_state;
mod main_window;
mod settings_window;
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
}

struct App {
    main_window: window::Id,
    settings_window: Option<window::Id>,

    app_settings: Settings,

    main_window_width: u32,
    main_window_height: u32,

    livesplit_state: LivesplitState,

    hotkeys: [HotkeyBox; 9],
    hotkey_focused: Option<usize>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        // let mut windows = BTreeMap::default();

        let (main_window, window_open_task) = window::open(window::Settings {
            exit_on_close_request: false,
            ..Default::default()
        });

        let settings = Settings::load().unwrap_or_default();

        (
            Self {
                main_window,
                settings_window: None,
                main_window_width: 0,
                main_window_height: 0,
                livesplit_state: LivesplitState::with_settings(&settings),
                app_settings: settings,

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
        if window == self.main_window {
            main_window::view(self)
        } else if self.settings_window == Some(window) {
            settings_window::view(self)
        } else {
            panic!("Tried to view untracked window")
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
                    .update(self.main_window_width, self.main_window_height)
            }
            Message::OpenSettingsWindow => {
                let (id, window_task) = window::open(window::Settings::default());

                self.settings_window = Some(id);

                self.livesplit_state.disable_hotkeys();

                settings_window::load_hotkeys_from_hks(&self.livesplit_state, &mut self.hotkeys);
                return window_task.discard();
            }
            Message::DiscardHotkeys => {
                settings_window::load_hotkeys_from_hks(&self.livesplit_state, &mut self.hotkeys);
            }
            Message::SaveHotkeys => {
                settings_window::save_hotkeys_to_hks(&mut self.livesplit_state, &self.hotkeys);
                self.livesplit_state
                    .save_hotkeys_to_settings(&mut self.app_settings);
            }
            Message::WindowClosed(window) => {
                if window == self.main_window {
                    self.app_settings.save();
                    return iced::exit();
                } else if self.settings_window == Some(window) {
                    self.settings_window = None;
                    self.hotkey_focused = None;
                    self.livesplit_state.enable_hotkeys();
                } else {
                    panic!("Tried to close untracked window")
                }
            }
            Message::KeyEvent(id, evt) => {
                if let keyboard::Event::KeyPressed {
                    physical_key,
                    modifiers,
                    ..
                } = evt
                {
                    if self.settings_window == Some(id) {
                        // todo filter on focus
                        self.hotkey_focused.inspect(|f| {
                            self.hotkeys[*f].value =
                                iced_key_to_livesplit_hotkey(physical_key, modifiers)
                        });
                    }
                }
            }
            Message::HotkeyBoxChangedFocus(id, focus) => {
                if focus {
                    self.hotkey_focused = Some(id)
                } else if self.hotkey_focused.is_some_and(|new_id| new_id == id) {
                    self.hotkey_focused = None
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
                    return Task::future(
                        rfd::AsyncMessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Failed to load splits")
                            .set_description(format!("Failed to load splits, got error {e}"))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show(),
                    )
                    .discard();
                } else {
                    self.app_settings.splits_path.replace(path);
                }
            }
            Message::TrySaveSplits => {
                return Task::future(Self::get_save_splits_path(None));
            }
            Message::SaveSplits(path) => {
                println!("Saving splits!");

                if let Err(e) = self.livesplit_state.save_splits(&path) {
                    return Task::future(
                        rfd::AsyncMessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Failed to save splits")
                            .set_description(format!("Failed to save splits, got error {e}"))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show(),
                    )
                    .discard();
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
                    return Task::future(
                        rfd::AsyncMessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Failed to load layout")
                            .set_description(format!("Failed to load layout, got error {e}"))
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show(),
                    )
                    .discard();
                } else {
                    self.app_settings.layout_path.replace(path)
                };
            }
            //
            Message::CloseRequested(window) => {
                if window == self.main_window {
                    let (close_window_task, ct) = window::close::<Message>(window).abortable();

                    let save_if_dirty_task = self.save_if_dirty(ct);

                    // disgusting hacky nonsense but i dont understand iced's async model enough to make it any better.
                    // ideally some kind of like combination of and_then and chain
                    // i feel like if i knew what a monad was i could make this cleaner
                    let (middle_task, ct) = save_if_dirty_task.chain(close_window_task).abortable();

                    let timer_running_task = if self.livesplit_state.is_timer_mid_run() {
                        Task::future(async move {
                            if let MessageDialogResult::No = AsyncMessageDialog::new()
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
                } else if self.settings_window == Some(window) {
                    return window::close::<Message>(window);
                } else {
                    panic!("Tried to close untracked window")
                }
            }
        }

        Task::none()
    }

    pub fn theme(&self, _: window::Id) -> Theme {
        Theme::Dark
    }

    pub fn title(&self, window: window::Id) -> String {
        if window == self.main_window {
            "LiveSplit".into()
        } else if self.settings_window == Some(window) {
            "Settings | LiveSplit".into()
        } else {
            panic!("Tried to get title of untracked window")
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
        // the compiler probably optimizes this nonsense out
        // TODO probably check
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
                };
                Message::None
            }
        }
    }

    fn save_if_dirty(&mut self, ct: iced::task::Handle) -> Task<Message> {
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
