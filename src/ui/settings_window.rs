use iced::border::Radius;
use iced::widget::{column, row};
use iced::{Border, Length, Padding};
use iced_aw::{grid, grid_row};
use iced_widget::container;

use crate::{App, Message, widgets::FocalWrapper};

pub fn view(app: &App) -> iced::Element<'_, Message> {
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
    container(column![hotkeys,])
        .padding(Padding::new(16.0))
        .into()
}
