use iced_wgpu::Renderer;
use iced_winit::{
    widget::{text_input, Column, Container, Text, TextInput},
    Color, Command, Element, Length, Padding, Program,
};

use crate::core::{command::ExecCommand, evbus::Sender};

pub struct Terminal {
    input: text_input::State,
    input_value: String,
    output: String,
    tx: Sender,
}

impl Terminal {
    pub fn new(tx: Sender) -> Self {
        Self {
            input: text_input::State::focused(),
            input_value: Default::default(),
            output: Default::default(),
            tx,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    InputSummit,
    Output(String),
}

impl Terminal {
    async fn execute(tx: Sender, input: String) -> String {
        let vec = input.split(' ').map(|s| s.to_string()).collect::<Vec<_>>();
        let (cmd, args) = vec.split_first().unwrap();

        match ExecCommand::post::<String>(&tx, cmd.clone(), args.to_vec()).await {
            Ok(o) => o.to_string(),
            Err(e) => e.to_string(),
        }
    }
}

impl Program for Terminal {
    type Renderer = Renderer;

    type Message = Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::InputChanged(v) => {
                self.input_value = v;
                Command::none()
            }
            Message::InputSummit => {
                self.output = "Waiting for...!".into();
                Command::perform(
                    Terminal::execute(self.tx.clone(), self.input_value.clone()),
                    Message::Output,
                )
            }
            Message::Output(v) => {
                self.output = v;
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message, Self::Renderer> {
        let editor = TextInput::new(
            &mut self.input,
            "Type CMD",
            &self.input_value,
            Message::InputChanged,
        )
        .width(Length::Fill)
        .size(48)
        .padding(Padding::from([8, 8]))
        .style(style::Editor {})
        .on_submit(Message::InputSummit);

        let output = Text::new(&self.output)
            .size(36)
            .width(Length::Fill)
            .height(Length::Units(64))
            .color(Color::WHITE);

        Container::new(Column::new().push(editor).spacing(8).push(output))
            .width(Length::Fill)
            .height(Length::Fill)
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
                border_width: 1.0,
                border_color: Color::WHITE,
                ..Default::default()
            }
        }

        fn focused(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border_width: 1.0,
                border_color: Color::WHITE,
                ..Default::default()
            }
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgba8(238, 238, 228, 0.5)
        }

        fn value_color(&self) -> Color {
            Color::WHITE
        }

        fn selection_color(&self) -> Color {
            Color::from_rgba8(238, 238, 228, 0.2)
        }
    }
}
