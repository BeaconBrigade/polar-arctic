use iced::{
    alignment, executor, text_input, Application, Column, Command, Element, Length, Subscription,
    Text, TextInput,
};
use std::time;
use plotters_iced::{Chart, ChartWidget, DrawingBackend};
use plotters::prelude::ChartBuilder;

mod blue;

#[derive(Default)]
pub struct Menu {
    device_input: text_input::State,
    device_id: String,
    sensor: Option<()>, // TODO - fill with actual arctic sensor when update is made
    ecg: EcgChart,
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Tick,
    InputChanged(String),
    CreateSensor,
}

impl Application for Menu {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Menu::default(), Command::none())
    }

    fn title(&self) -> String {
        "Polar-Arctic".to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => {}
            Message::Tick => {}
            Message::InputChanged(msg) => {
                self.device_id = msg;
            }
            Message::CreateSensor => {
                // Construct sensor
                println!("Sensor created!");
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(time::Duration::from_millis(16)).map(|_| Message::Tick)
    }

    fn view(&mut self) -> Element<'_, Message> {
        let title = Text::new("Polar-Arctic")
            .width(Length::Fill)
            .size(60)
            .horizontal_alignment(alignment::Horizontal::Center);

        let input = TextInput::new(
            &mut self.device_input,
            "Device ID",
            &self.device_id,
            Message::InputChanged,
        )
            .padding(15)
            .size(25)
            .on_submit(Message::CreateSensor);

        let testing = Text::new(&self.device_id).size(25).width(Length::Fill);

        Column::new()
            .spacing(20)
            .width(Length::Fill)
            .max_width(1000)
            .push(title)
            .push(input)
            .push(testing)
            .into()
    }
}

#[derive(Default)]
pub struct EcgChart;

impl Chart<Message> for EcgChart {
    fn build_chart<DB: DrawingBackend>(&self, builder: ChartBuilder<DB>) {}
}

impl EcgChart {
    pub fn new() -> EcgChart {
        Self
    }

    fn view(&mut self) -> Element<Message> {
        let chart = ChartWidget::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        chart.into()
    }
}
