use iced_wgpu::Renderer;
use iced_winit::{
    alignment,
    widget::{container, text_input, Column, Container, Row, Text, TextInput},
    Alignment, Command, Element, Font, Length, Padding, Program,
};

pub struct Terminal {
    input: text_input::State,
    input_value: String,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            input_value: "".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
}

impl Program for Terminal {
    type Renderer = Renderer;

    type Message = Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::InputChanged(v) => self.input_value = v,
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message, Self::Renderer> {
        let editor = TextInput::new(
            &mut self.input,
            "Type CMD",
            &self.input_value,
            Message::InputChanged,
        )
        .size(48)
        .font(Font::External {
            name: "Hack",
            bytes: include_bytes!("assets/Hack-Regular.ttf"),
        })
        .style(style::Editor {});

        Container::new(editor)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([10, 10]))
            .into()
    }
}

mod style {
    use iced_winit::{widget::text_input, Background, Color};

    pub struct Editor {}
    impl text_input::StyleSheet for Editor {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                ..Default::default()
            }
        }

        fn focused(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                ..Default::default()
            }
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgba8(238, 238, 228, 0.5)
        }

        fn value_color(&self) -> Color {
            Color::from_rgba8(238, 238, 228, 1.0)
        }

        fn selection_color(&self) -> Color {
            Color::from_rgba8(238, 238, 228, 0.2)
        }
    }
}
