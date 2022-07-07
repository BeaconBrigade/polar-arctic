use csv::ReaderBuilder;
use iced::pure::{button, column, row, text_input, widget::Text, Pure, State};
use iced::{Column, Length, Rule, Row};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend};
use std::{collections::VecDeque, io};
use std::fmt::Write as _;

use crate::menu::Paths;

use super::{blue::fs::update_recent, modal::PopupMessage, Message, WhichView};

pub struct Data {
    chart: EcgChart,
    device_id: String,
    state: State,
    recent_data: Recent,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            chart: EcgChart::new("output/ecg.csv".to_string()).unwrap(),
            device_id: "".to_string(),
            state: State::new(),
            recent_data: Recent::default(),
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&mut self) -> iced::Element<Message> {
        let back = button(Text::new("Back to menu").size(20))
            .on_press(Message::SwitchView(WhichView::Menu));
        let help =
            button(Text::new("Help").size(20)).on_press(Message::Popup(PopupMessage::DataHelp));

        let header = row().push(back).push(help);

        let input = text_input("Device ID", &self.device_id, Message::NewDeviceID)
            .padding(15)
            .size(20)
            .on_submit(Message::CreateSensor);

        let stop_button = button(Text::new("Stop Measurement")).on_press(Message::StopMeasurement);

        let view = column()
            .spacing(20)
            .width(Length::Fill)
            .max_width(1000)
            .push(header)
            .push(Rule::horizontal(10))
            .push(input)
            .push(stop_button);

        let pure = Pure::new(&mut self.state, view);

        // I hate doing this
        let mut rr_text = String::new();
        let mut first = true;
        for item in self.recent_data.rr.iter() {
            if !first {
                rr_text.push_str(", ");
            }
            let _ = write!(rr_text, "{}", item);
            first = false;
        }

        let bpm = iced::Text::new(&format!("Heart rate (BPM): {}", self.recent_data.bpm));
        let rr = iced::Text::new(&format!("RR interval (µV): {}", rr_text));
        let acc_title = iced::Text::new("Acceleration (mG):");
        let x = iced::Text::new(&format!("    X: {}", self.recent_data.x));
        let y = iced::Text::new(&format!("    Y: {}", self.recent_data.y));
        let z = iced::Text::new(&format!("    Z: {}", self.recent_data.z));

        let data_column = Column::new()
            .spacing(20)
            .push(bpm)
            .push(rr)
            .push(acc_title)
            .push(x)
            .push(y)
            .push(z);

        let data = Row::new()
            .spacing(20)
            .push(self.chart.view())
            .push(data_column);

        Column::new()
            .spacing(20)
            .push(pure)
            .push(data)
            .into()
    }

    pub fn update_id(&mut self, msg: String) {
        self.device_id = msg;
    }

    pub fn id(&mut self) -> &mut String {
        &mut self.device_id
    }

    pub fn update(&mut self, paths: Paths) -> Result<(), io::Error> {
        self.chart.update();
        self.recent_data.update(paths)
    }

    pub fn set_path(&mut self, path: String) {
        self.chart.path = path
    }
}

// Store chart data
#[derive(Default)]
struct EcgChart {
    data_points: VecDeque<(u64, i32)>,
    pub path: String,
}

impl EcgChart {
    pub fn new(path: String) -> Result<EcgChart, Box<dyn std::error::Error>> {
        let mut chart = Self {
            data_points: VecDeque::with_capacity(200),
            path,
        };
        chart.init_data()?;

        Ok(chart)
    }

    // Draw chart
    fn view(&mut self) -> iced::Element<Message> {
        let chart = ChartWidget::new(self)
            .width(Length::Units(400))
            .height(Length::Units(400));

        chart.into()
    }

    // Get initial data from file
    fn init_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_path(&self.path)
            .unwrap();

        let records = rdr
            .records()
            .skip(1)
            .collect::<Vec<Result<csv::StringRecord, csv::Error>>>()
            .into_iter()
            .rev()
            .take(200)
            .rev();

        // skip extra header row
        for record in records {
            let result = record?;
            self.push((result[0].parse()?, result[1].parse::<i32>()?));
        }
        Ok(())
    }

    // Add to back and pop off front
    fn push(&mut self, val: (u64, i32)) {
        self.data_points.push_front(val);
        while self.data_points.len() > 200 {
            self.data_points.pop_back();
        }
    }

    // Update data - remove data from the end and add to the start
    fn update(&mut self) {
        if let Err(e) = self.init_data() {
            eprintln!("Error getting data: {}", e);
        }
    }
}

impl Chart<Message> for EcgChart {
    // Create plotters chart
    fn build_chart<DB: DrawingBackend>(&self, mut builder: ChartBuilder<DB>) {
        let mut ctx = builder
            .set_label_area_size(LabelAreaPosition::Bottom, -181)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .caption("ECG Data", ("sans-serif", 30u32))
            .build_cartesian_2d(0..200u64, -1000..1000i32)
            .unwrap();

        ctx.configure_mesh()
            .set_tick_mark_size(LabelAreaPosition::Bottom, 5)
            .draw()
            .unwrap();

        let series = LineSeries::new(
            (0..self.data_points.len() as u64).map(|p| (p, self.data_points[p as usize].1)),
            &BLACK,
        );

        ctx.draw_series(series).expect("Error making graph");
    }
}

#[derive(Debug, Default, Clone)]
pub struct Recent {
    pub bpm: u8,
    pub rr: Vec<u16>,
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Recent {
    pub fn update(&mut self, paths: Paths) -> Result<(), io::Error> {
        update_recent(self, paths)
    }
}
