pub trait FocalWrapper<F: Fn(bool) -> Message, Message: Clone> {
    type Wrapped;
    fn wrap_focus(self, focus_changed: F) -> Self::Wrapped;
}

impl<'a, M, R, T, F> FocalWrapper<F, M> for iced::widget::TextInput<'a, M, T, R>
where
    T: iced::widget::text_input::Catalog,
    R: iced::advanced::text::Renderer,
    M: Clone,
    F: Fn(bool) -> M,
{
    type Wrapped = TextInputFocalWrapper<'a, M, T, R, F>;

    fn wrap_focus(self, focused_change: F) -> Self::Wrapped {
        TextInputFocalWrapper::new(self, focused_change)
    }
}

pub struct TextInputFocalWrapper<'a, Message, Theme, Renderer, F>
where
    Theme: iced::widget::text_input::Catalog,
    Renderer: iced::advanced::text::Renderer,
    Message: Clone,
    F: Fn(bool) -> Message,
{
    content: iced::widget::TextInput<'a, Message, Theme, Renderer>,
    f: F,
}

impl<'a, Message, Theme, Renderer, F> TextInputFocalWrapper<'a, Message, Theme, Renderer, F>
where
    Theme: iced::widget::text_input::Catalog,
    Renderer: iced::advanced::text::Renderer,
    Message: Clone,
    F: Fn(bool) -> Message,
{
    pub fn new(content: iced::widget::TextInput<'a, Message, Theme, Renderer>, f: F) -> Self {
        Self { content, f }
    }
}
impl<'a, Message, Theme, Renderer, F> iced::advanced::Widget<Message, Theme, Renderer>
    for TextInputFocalWrapper<'a, Message, Theme, Renderer, F>
where
    Theme: iced::widget::text_input::Catalog,
    Renderer: iced::advanced::text::Renderer,
    Message: Clone,
    F: Fn(bool) -> Message,
{
    fn size(&self) -> iced::Size<iced::Length> {
        iced::advanced::widget::Widget::size(&self.content)
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        iced::advanced::widget::Widget::layout(&self.content, tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        iced::advanced::widget::Widget::draw(
            &self.content,
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        let focused_before = is_focused::<Renderer>(tree);

        let res = iced::advanced::widget::Widget::on_event(
            &mut self.content,
            tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let focused_after = is_focused::<Renderer>(tree);

        if focused_before != focused_after {
            shell.publish((self.f)(focused_after));
        }
        return res;
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        iced::advanced::widget::Widget::size_hint(&self.content)
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::Widget::tag(&self.content)
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::Widget::state(&self.content)
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        iced::advanced::widget::Widget::children(&self.content)
    }

    fn diff(&self, _tree: &mut iced::advanced::widget::Tree) {
        iced::advanced::widget::Widget::diff(&self.content, _tree)
    }

    fn operate(
        &self,
        state: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        iced::advanced::widget::Widget::operate(&self.content, state, layout, renderer, operation)
    }

    fn mouse_interaction(
        &self,
        state: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        iced::advanced::widget::Widget::mouse_interaction(
            &self.content,
            state,
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}

fn is_focused<Renderer: iced::advanced::text::Renderer>(
    tree: &mut iced::advanced::widget::Tree,
) -> bool {
    let state = tree
        .state
        .downcast_ref::<iced::widget::text_input::State<Renderer::Paragraph>>();

    state.is_focused()
}

impl<'a, Message, Theme, Renderer, F>
    std::convert::From<TextInputFocalWrapper<'a, Message, Theme, Renderer, F>>
    for iced::Element<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::text_input::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer + 'a,
    Message: Clone + 'a,
    F: Fn(bool) -> Message + 'a,
{
    fn from(value: TextInputFocalWrapper<'a, Message, Theme, Renderer, F>) -> Self {
        Self::new(value)
    }
}
