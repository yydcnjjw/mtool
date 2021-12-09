use iced_wgpu::Renderer;
use iced_winit::{
    alignment,
    widget::{container, text_input, Column, Container, Row, Text, TextInput},
    Alignment, Color, Command, Element, Font, Length, Padding, Program, Widget,
};

use crate::core::{command::ExecCommand, evbus::Sender};

pub struct Terminal {
    tx: Sender,
    input: text_input::State,
    input_value: String,
    output: String,
}

impl Terminal {
    pub fn new(tx: Sender) -> Self {
        Self {
            tx,
            input: text_input::State::focused(),
            input_value: Default::default(),
            output: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    InputSummit,
    Output(String),
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
                self.output = self.input_value.clone();
                Command::none()
            }
            // Command::perform(
            // {
            //     let tx = self.tx.clone();

            //     let (cmd, args) = self
            //         .input_value
            //         .split(' ')
            //         .map(|s| s.to_string())
            //         .collect::<Vec<_>>()
            //         .split_first()
            //         .unwrap();

            //     // async { ExecCommand::post(&tx, cmd.clone(), args.to_vec()).await }
            //     println!("-----perform output: {}", self.output);
            //     let output = self.input_value.clone();
            //     async { output }
            // },
            // |v| {
            //     println!("-----callback output: {}", v);
            //     Message::Output(v)
            // },
            // ),
            Message::Output(v) => {
                self.output = v;
                println!("-----output: {}", self.output);
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
            Color::from_rgba8(238, 238, 228, 1.0)
        }

        fn selection_color(&self) -> Color {
            Color::from_rgba8(238, 238, 228, 0.2)
        }
    }
}
