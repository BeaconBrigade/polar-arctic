use csv::ReaderBuilder;
use iced::{
    alignment, executor, text_input, Application, Column, Command, Element, Length, Subscription,
    Text, TextInput, Row, Rule, Button, button,
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend};
use std::collections::VecDeque;

use std::time;
use chrono::{DateTime, Utc};

mod blue;

#[derive(Default)]
pub struct Menu {
    device_input: text_input::State,
    device_id: String,
    sensor: Option<()>, // TODO - fill with actual arctic sensor when update is made
    chart: EcgChart,
    meta_state: MetaState,
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Tick,
    InputChanged(String),
    CreateSensor,
    NewMeta,
    ChangeMeta(WhichMeta, String),
}

impl Application for Menu {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let chart = EcgChart::new().expect("Error making graph.");

        (
            Menu {
                chart,
                ..Default::default()
            },
            Command::none(),
        )
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
            Message::NewMeta => {}
            Message::ChangeMeta(which, msg) => {
                match which {
                    WhichMeta::Id => self.meta_state.meta_data.id = msg,
                    WhichMeta::Session => self.meta_state.meta_data.session = msg,
                    WhichMeta::Trial => self.meta_state.meta_data.trial = msg,
                    WhichMeta::Description => self.meta_state.meta_data.description = msg,
                }
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

        // Data side
        let left = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .max_width(1000)
            .push(input)
            .push(testing)
            .push(self.chart.view());


        let meta_header = Text::new("Meta")
            .width(Length::Fill)
            .size(40);

        // meta side
        let right = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .max_width(1000)
            .push(meta_header)
            .push(self.meta_state.view());


        // full view
        let body = Row::new()
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(left)
            .push(Rule::vertical(2))
            .push(right);

        Column::new()
            .push(title)
            .push(body)
            .into()
    }
}

// Store chart data
#[derive(Default)]
struct EcgChart {
    data_points: VecDeque<(u64, f32)>,
}

impl EcgChart {
    pub fn new() -> Result<EcgChart, Box<dyn std::error::Error>> {
        let mut chart = Self::default();
        chart.init_data()?;

        Ok(chart)
    }

    // Draw chart
    fn view(&mut self) -> Element<Message> {
        let chart = ChartWidget::new(self)
            .width(Length::Units(400))
            .height(Length::Units(400));

        chart.into()
    }

    // Get initial data from file
    fn init_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_path("output/test.csv")
            .unwrap();

        // skip extra header row
        for record in rdr.records().skip(1) {
            let result = record?;
            self.data_points
                .push_back((result[0].parse()?, result[1].parse::<f32>()? * 100.0));
        }

        Ok(())
    }

    // Update data - remove data from the end and add to the start
    fn update(&mut self) {}
}

impl Chart<Message> for EcgChart {
    // Create plotters chart
    fn build_chart<DB: DrawingBackend>(&self, mut builder: ChartBuilder<DB>) {
        let start = self.data_points[0].0;
        let end = self.data_points.back().unwrap().0;

        let mut ctx = builder
            .set_label_area_size(LabelAreaPosition::Bottom, -181)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .caption("ECG Data", ("sans-serif", 30u32))
            .build_cartesian_2d(0..100u64, -1000.0..1000.0f64)
            .unwrap();

        ctx.configure_mesh()
            .set_tick_mark_size(LabelAreaPosition::Bottom, 5)
            .draw()
            .unwrap();

        ctx.draw_series(LineSeries::new(
            (start..end).map(|p| (p, self.data_points[(p - start) as usize].1 as f64)),
            &BLACK,
        ))
        .expect("Error making graph");
    }
}

// Store meta-data about this run
struct Meta {
    pub id: String,
    pub session: String,
    pub trial: String,
    pub description: String,
    pub date: DateTime<Utc>,
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            id: "".to_string(),
            session: "".to_string(),
            trial: "".to_string(),
            description: "".to_string(),
            date: Utc::now(),
        }
    }
}

// Which kind of metadata to change 
#[derive(Debug, Clone, Copy)]
pub enum WhichMeta {
    Id,
    Session,
    Trial,
    Description,
}

// Store states for meta data
#[derive(Default)]
struct MetaState {
    id_state: text_input::State,
    session_state: text_input::State,
    trial_state: text_input::State,
    description_state: text_input::State,
    submit_state: button::State,
    pub meta_data: Meta,
}

impl MetaState {
    fn view(&mut self) -> Element<Message> {
        let id = TextInput::new(
            &mut self.id_state,
            "Participant ID",
            &self.meta_data.id,
            |s| Message::ChangeMeta(WhichMeta::Id, s),
        );

        let session = TextInput::new(
            &mut self.session_state,
            "Session Number",
            &self.meta_data.session,
            |s| Message::ChangeMeta(WhichMeta::Session, s),
        );

        let trial = TextInput::new(
            &mut self.trial_state,
            "Trial number",
            &self.meta_data.trial,
            |s| Message::ChangeMeta(WhichMeta::Trial, s),
        );

        let description = TextInput::new(
            &mut self.description_state,
            "Description/Notes",
            &self.meta_data.description,
            |s| Message::ChangeMeta(WhichMeta::Description, s),
        );

        let submit = Button::new(
            &mut self.submit_state, 
            Text::new("Submit"),
        );

        Column::new()
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(id)
            .push(session)
            .push(trial)
            .push(description)
            .push(submit)
            .into()
    }
}
