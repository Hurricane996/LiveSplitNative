use iced::{Border, Length, Padding, alignment::Horizontal};
use iced_aw::{grid, grid_row};
use iced_widget::{button, column, container, mouse_area, row, scrollable, text, text_input};
use livesplit_core::run::editor::SegmentState;

trait BoolAsSome {
    fn as_some<T>(&self, t: T) -> Option<T>;
}

impl BoolAsSome for bool {
    fn as_some<T>(&self, t: T) -> Option<T> {
        if *self { Some(t) } else { None }
    }
}
use crate::{
    App,
    state::splits_editor::{Message, SplitsEditorState},
    widgets::FocalWrapper,
};

pub fn view(app: &App) -> iced::Element<'_, crate::Message> {
    let splits_editor_state = app
        .splits_editor_state
        .as_ref()
        .expect("Tried to draw edit splits window with no splits editor");

    let editor_state = &splits_editor_state.editor_state;

    let game_tb = text_input("", &editor_state.game)
        .on_input(|x| Message::UpdateGameName(x).into_app_message());
    let category_tb = text_input("", &editor_state.category)
        .on_input(|x| Message::UpdateCategoryName(x).into_app_message());

    let start_timer_at_tb = text_input("", &splits_editor_state.offset_buffer)
        .on_input(|x| Message::UpdateOffsetBuffer(x).into_app_message())
        .wrap_focus(|f| {
            if f {
                crate::Message::None
            } else {
                Message::OffsetTextboxBlur.into_app_message()
            }
        });
    let attempts_tb = text_input(
        "",
        &format!("{}", splits_editor_state.editor.attempt_count()),
    )
    .on_input(|x| Message::UpdateNumAttempts(x).into_app_message());

    let game_info = grid![
        grid_row![text("Game"), text("Category")],
        grid_row![game_tb, category_tb],
        grid_row![text("Start Timer At"), text("Attempts")],
        grid_row![start_timer_at_tb, attempts_tb]
    ]
    .padding(8.);

    let grid = {
        let column_width = Length::Fill;

        let header = row![
            text("Segment Name")
                .width(column_width)
                .align_x(Horizontal::Center),
            text("Split Time")
                .width(column_width)
                .align_x(Horizontal::Center),
            text("Segment Time")
                .width(column_width)
                .align_x(Horizontal::Center),
            text("Best Segment")
                .width(column_width)
                .align_x(Horizontal::Center),
        ];

        container(column![
            header,
            scrollable(column(editor_state.segments.iter().enumerate().map(
                |(idx, segment)| table_row(idx, segment, column_width, splits_editor_state)
            )))
            .style(|t, s| {
                let mut s = scrollable::default(t, s);
                s.container.border = Border {
                    color: t.extended_palette().background.base.text,
                    width: 1.,
                    radius: Default::default(),
                };
                s
            })
        ])
        .style(|t| container::Style {
            border: Border {
                color: t.extended_palette().background.base.text,
                width: 1.,
                radius: Default::default(),
            },
            ..Default::default()
        })
    };

    let buttons = column![
        button("Insert Above")
            .width(Length::Fill)
            .on_press(Message::InsertAboveClicked.into_app_message()),
        button("Insert Below")
            .width(Length::Fill)
            .on_press(Message::InsertBelowClicked.into_app_message()),
        button("Remove Segment").width(Length::Fill).on_press_maybe(
            editor_state
                .buttons
                .can_remove
                .as_some(Message::RemoveSegmentClicked.into_app_message())
        ),
        button("Move Up").width(Length::Fill).on_press_maybe(
            editor_state
                .buttons
                .can_move_up
                .as_some(Message::MoveUpClicked.into_app_message())
        ),
        button("Move Down").width(Length::Fill).on_press_maybe(
            editor_state
                .buttons
                .can_move_down
                .as_some(Message::MoveDownClicked.into_app_message())
        ),
    ]
    .width(175.);

    let splits_section = row![buttons, grid].spacing(8.);
    column![
        game_info,
        container(splits_section).padding(Padding::new(8.))
    ]
    .spacing(8.)
    .into()
}
fn table_row<'a, R>(
    index: usize,
    segment: &SegmentState,
    column_width: Length,
    splits_editor_state: &SplitsEditorState,
) -> iced::Element<'a, crate::Message, iced::Theme, R>
where
    R: iced::advanced::text::Renderer + 'a,
{
    fn gold_style(t: &iced::Theme, s: iced::widget::text_input::Status) -> text_input::Style {
        let mut style = text_input::default(t, s);

        style.value = iced::Color::from_rgb8(224, 194, 0);

        style
    }

    let segment_style = if segment.segment_time == segment.best_segment_time {
        gold_style
    } else {
        text_input::default
    };

    let segment_name = text_input("", &segment.name)
        .on_input(|text| Message::UpdateSegmentName(text).into_app_message())
        .width(column_width)
        .wrap_focus(move |f| {
            if f {
                Message::SelectRow(index).into_app_message()
            } else {
                crate::Message::None
            }
        });
    let split_time = text_input("", &splits_editor_state.split_time_buffers[index])
        .on_input(move |text| Message::UpdateSplitTimeBuffer(text, index).into_app_message())
        .width(column_width)
        .wrap_focus(move |f| {
            if f {
                Message::SelectRow(index)
            } else {
                Message::SplitTimeBlur(index)
            }
            .into_app_message()
        });
    let segment_time = text_input("", &splits_editor_state.segment_time_buffers[index])
        .on_input(move |text| Message::UpdateSegmentTimeBuffer(text, index).into_app_message())
        .width(column_width)
        .style(segment_style)
        .wrap_focus(move |f| {
            if f {
                Message::SelectRow(index)
            } else {
                Message::SegmentTimeBlur(index)
            }
            .into_app_message()
        });
    let best_segment_time = text_input("", &splits_editor_state.best_segment_time_buffers[index])
        .on_input(move |text| Message::UpdateBestSegmentTimeBuffer(text, index).into_app_message())
        .width(column_width)
        .wrap_focus(move |f| {
            if f {
                Message::SelectRow(index)
            } else {
                Message::BestSegmentTimeBlur(index)
            }
            .into_app_message()
        });
    mouse_area(
        container(
            row![segment_name, split_time, segment_time, best_segment_time,]
                .spacing(8.)
                .padding(4.),
        )
        .style(if segment.selected.is_selected_or_active() {
            active_row_style
        } else if index % 2 == 0 {
            even_row_style
        } else {
            odd_row_style
        }),
    )
    .on_press(Message::SelectRow(index).into_app_message())
    .into()
}

fn odd_row_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.extended_palette().background.base.text),
        background: Some(iced::Background::Color(
            theme.extended_palette().background.base.color,
        )),
        border: Border {
            color: theme.extended_palette().background.base.text,
            width: 1.,
            radius: Default::default(),
        },
        ..Default::default()
    }
}

fn even_row_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.extended_palette().background.weak.text),
        background: Some(iced::Background::Color(
            theme.extended_palette().background.weak.color,
        )),
        border: Border {
            color: theme.extended_palette().background.base.text,
            width: 1.,
            radius: Default::default(),
        },
        ..Default::default()
    }
}

fn active_row_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.extended_palette().primary.weak.text),
        background: Some(iced::Background::Color(
            theme.extended_palette().primary.weak.color,
        )),
        border: Border {
            color: theme.extended_palette().background.base.text,
            width: 1.,
            radius: Default::default(),
        },
        ..Default::default()
    }
}
