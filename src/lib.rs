use iced::{
    alignment, executor, text_input, Application, Column, 
    Command, Element, Length, Subscription, Text, TextInput,
};
use std::time;
use plotters_iced::{Chart, ChartWidget, DrawingBackend};
use plotters::prelude::*;
use std::collections::VecDeque;
use csv::ReaderBuilder;

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
        let mut menu = Menu::default();
        menu.ecg.init_data().expect("Error loading data");

        (menu, Command::none())
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
            .push(self.ecg.view())
            .into()
    }
}

#[derive(Default)]
pub struct EcgChart {
    data_points: VecDeque<(u64, f32)>
}

impl Chart<Message> for EcgChart {
    
    fn build_chart<DB: DrawingBackend>(&self, mut builder: ChartBuilder<DB>) {
        let start = self.data_points[0].0;
        let end = self.data_points.back().unwrap().0;

        let mut ctx = builder
            .set_label_area_size(LabelAreaPosition::Top, -1i32)
            .set_label_area_size(LabelAreaPosition::Left, 40i32)
            .caption("ECG Data", ("sans-serif", 30u32))
            .build_cartesian_2d(0..100u64, -1000.0..1000.0f64)
            .unwrap();

        ctx.configure_mesh()
            .draw()
            .unwrap();

        ctx.draw_series(
            LineSeries::new(
                (start..end).map(|p| (p, self.data_points[(p - start) as usize].1 as f64)),
                &BLACK
        )).expect("Error making graph");
    }
}

impl EcgChart {
    pub fn new() -> EcgChart {
        EcgChart::default()
    }

    fn view(&mut self) -> Element<Message> {
        let chart = ChartWidget::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        chart.into()
    }

    fn init_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = ReaderBuilder::new().from_path("output/test.csv").unwrap();

        for record in rdr.records() {
            let result = record?;
            self.data_points.push_back((result[0].parse()?, result[1].parse::<f32>()? * 100.0));
        }

        Ok(())
    }
}
