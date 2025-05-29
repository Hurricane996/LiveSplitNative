use iced::{
    Border, Length, Theme,
    border::Radius,
    widget::{column, container, image},
};

use crate::{App, Message};

pub fn view(app: &App) -> iced::Element<'_, Message> {
    let (width, height) = app.livesplit_state.last_rendered_size();
    // TODO figure out how to avoid this clone
    let im = iced::widget::image(image::Handle::from_rgba(
        width,
        height,
        app.livesplit_state.view(),
    ));

    let container = container(im);

    iced_aw::ContextMenu::new(container, || {
        let style = |t: &Theme, _| iced::widget::button::Style {
            background: Some(iced::Background::Color(t.palette().background)),
            text_color: t.palette().text,
            border: Border {
                color: t.palette().text,
                width: 1.0,
                radius: Radius::new(0.),
            },
            ..Default::default()
        };

        column![
            iced::widget::button("Load Splits")
                .on_press(Message::TryLoadSplits)
                .style(style)
                .width(Length::Fill),
            iced::widget::button("Save Splits")
                .on_press(Message::TrySaveSplits)
                .style(style)
                .width(Length::Fill),
            iced::widget::button("EditSplits")
                .on_press(Message::OpenEditSplitsWindow)
                .style(style)
                .width(Length::Fill),
            iced::widget::button("Load Layout")
                .on_press(Message::TryLoadLayout)
                .style(style)
                .width(Length::Fill),
            iced::widget::button("Settings")
                .on_press(Message::OpenSettingsWindow)
                .style(style)
                .width(Length::Fill)
        ]
        .width(150.)
        .into()
    })
    .into()
}
