use csv::ReaderBuilder;
use iced::{
    alignment, executor, text_input, Application, Column, Command, Element, Length, Subscription,
    Text, TextInput, Row, Rule,
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend};
use std::collections::VecDeque;
use std::time;

mod blue;

#[derive(Default)]
pub struct Menu {
    device_input: text_input::State,
    device_id: String,
    sensor: Option<()>, // TODO - fill with actual arctic sensor when update is made
    chart: EcgChart,
    meta: Option<Meta>,
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Tick,
    InputChanged(String),
    CreateSensor,
    NewMeta,
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
            .push(meta_header);


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
    pub first_name: String,
    pub last_name: String,
    pub group_number: i8,
}

// Store errors converting input to meta data
enum ParseError {
    First,
    Last,
    GroupNumber,
}

impl TryFrom<[&str; 3]> for Meta {
    type Error = ParseError;

    fn try_from(data: [&str; 3]) -> Result<Self, Self::Error> {
        Ok(Self {
            first_name: if !data[0].is_empty() {
                data[0].to_owned()
            } else {
                return Err(ParseError::First);
            },
            last_name: if !data[1].is_empty() {
                data[1].to_owned()
            } else {
                return Err(ParseError::Last);
            },
            group_number: data[2].parse().map_err(|_| ParseError::GroupNumber)?,
        })
    }
}
