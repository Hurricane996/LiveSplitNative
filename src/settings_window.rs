use iced::border::Radius;
use iced::widget::{column, row};
use iced::{Border, Length, Padding};
use iced_aw::{grid, grid_row};
use iced_widget::container;
use livesplit_core::{HotkeyConfig, hotkey::Hotkey};

use crate::{App, Message, livesplit_state::LivesplitState, widgets::FocalWrapper};

pub struct HotkeyBox {
    label: &'static str,
    pub value: Option<Hotkey>,
}

impl HotkeyBox {
    pub fn new(label: &'static str) -> Self {
        Self { label, value: None }
    }
}
// "Start/Split",
// "Reset",
// "Undo",
// "Skip",
// "Pause",
// "Undo All Pauses",
// "Previous Comparison",
// "Next Comparison",
// "Toggle Timing Method",
pub fn load_hotkeys_from_hks(livesplit_state: &LivesplitState, hotkeys: &mut [HotkeyBox; 9]) {
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

pub fn save_hotkeys_to_hks(livesplit_state: &mut LivesplitState, hotkeys: &[HotkeyBox; 9]) {
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

    livesplit_state
        .hks
        .set_config(HotkeyConfig {
            split: None,
            reset: None,
            undo: None,
            skip: None,
            pause: None,
            undo_all_pauses: None,
            previous_comparison: None,
            next_comparison: None,
            toggle_timing_method: None,
        })
        .expect("Failed to save config");

    livesplit_state
        .hks
        .set_config(config)
        .expect("Failed to save config")
}
pub fn view<'a>(app: &'a App) -> iced::Element<'a, Message> {
    let hotkeys = container(
        column![
            iced::widget::text("Hotkeys: "),
            container(
                grid(
                    app.hotkeys
                        .iter()
                        .enumerate()
                        .map(|(index, hotkey)| {
                            let hotkey_text =
                                hotkey.value.map(|x| x.to_string()).unwrap_or_default();
                            grid_row![
                                iced::widget::text(hotkey.label),
                                iced::widget::text_input("", &hotkey_text)
                                    .on_input(|_| Message::None)
                                    .width(Length::Fill)
                                    .wrap_focus(move |is_focused| {
                                        Message::HotkeyBoxChangedFocus(index, is_focused)
                                    }),
                                iced::widget::button("Clear").on_press(Message::ClearHotkey(index))
                            ]
                            .into()
                        })
                        .collect(),
                )
                .width(Length::Fill)
                .column_widths(&[Length::Shrink, Length::Fill, Length::Shrink])
                .row_spacing(8.)
                .column_spacing(8.)
            )
            .padding(Padding::default().left(16.)),
            row![
                iced::widget::button("Save").on_press(Message::SaveHotkeys),
                iced::widget::button("Discard").on_press(Message::DiscardHotkeys)
            ]
            .spacing(8.)
        ]
        .spacing(8.),
    )
    .style(|theme: &iced::Theme| container::Style {
        border: Border {
            color: theme.palette().text,
            width: 1.,
            radius: Radius::default(),
        },
        ..Default::default()
    })
    .padding(Padding::default().left(16.0).top(8.).bottom(8.));
    // todo grid this
    container(column![hotkeys,])
        .padding(Padding::new(16.0))
        .into()
}
