use iced::pure::{
    widget::{Button, Column, Row, Text, TextInput},
    Element,
};
use iced::{Length, Rule};
use plotters::prelude::*;
use plotters_iced::pure::{Chart, ChartWidget};
use rev_lines::RevLines;
use std::fs::File;
use std::io::BufReader;
use std::{collections::VecDeque, path::PathBuf};
use tokio::sync::watch::Receiver;

use super::{modal::PopupMessage, Message, WhichView};

#[derive(Debug)]
pub struct Data {
    chart: EcgChart,
    device_id: String,
    recent_data: Recent,
    receiver: Option<DataReceiver>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            chart: EcgChart::new().unwrap(),
            device_id: String::default(),
            recent_data: Recent::default(),
            receiver: None,
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_receivers(&mut self, receiver: DataReceiver) {
        self.receiver = Some(receiver);
    }

    pub fn view(&self) -> Element<Message> {
        let back = Button::new(Text::new("Back to menu").size(20))
            .on_press(Message::SwitchView(WhichView::Menu))
            .padding(10);
        let help = Button::new(Text::new("Help").size(20))
            .on_press(Message::Popup(PopupMessage::DataHelp))
            .padding(10);

        let header = Row::new().push(back).push(help);

        let input = TextInput::new("Device ID", &self.device_id, Message::NewDeviceID)
            .padding(10)
            .size(20)
            .on_submit(Message::CreateSensor);

        let stop_button =
            Button::new(Text::new("Stop Measurement")).on_press(Message::StopMeasurement);

        let view = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .max_width(1000)
            .push(header)
            .push(Rule::horizontal(10))
            .push(input)
            .push(stop_button);

        let rr_text = &self.recent_data.rr;
        let mut rr_text = rr_text.chars();
        rr_text.next();

        let bpm = iced::Text::new(&format!("Heart rate (BPM): {}", self.recent_data.bpm));
        let rr = iced::Text::new(&format!(
            "RR interval (µV): {}",
            rr_text.as_str().replace(',', ", ")
        ));
        let acc_title = Text::new("Acceleration (mG):");
        let x = Text::new(&format!("    X: {}", self.recent_data.x));
        let y = Text::new(&format!("    Y: {}", self.recent_data.y));
        let z = Text::new(&format!("    Z: {}", self.recent_data.z));

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
            .push(view)
            .push(self.chart.view())
            .push(data_column);

        Column::new().spacing(20).push(data).into()
    }

    pub fn update_id(&mut self, msg: String) {
        self.device_id = msg;
    }

    pub fn id(&mut self) -> &mut String {
        &mut self.device_id
    }

    pub fn update(&mut self) {
        self.chart.update_data();
        if let Some(rx) = &self.receiver {
            self.recent_data.bpm = rx.hr();
            self.recent_data.rr = rx.rr();
            let (x, y, z) = rx.acc();
            self.recent_data.x = x;
            self.recent_data.y = y;
            self.recent_data.z = z;
        }
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        self.chart.path = path;
    }
}

// Store chart data
#[derive(Default, Debug)]
struct EcgChart {
    data_points: VecDeque<(u64, i32)>,
    pub path: Option<PathBuf>,
}

impl EcgChart {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut chart = Self {
            data_points: VecDeque::with_capacity(200),
            path: None,
        };
        chart.update_data_internal()?;

        Ok(chart)
    }

    // Draw chart
    fn view(&self) -> Element<Message> {
        let chart = ChartWidget::new(self)
            .width(Length::Units(400))
            .height(Length::Units(400));

        chart.into()
    }

    // Get initial data from file
    fn update_data_internal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = if let Some(path) = &self.path {
            path
        } else {
            return Ok(());
        };
        let file = File::open(path)?;
        let records = RevLines::with_capacity(400, BufReader::new(file))?;

        for record in records.take(200) {
            if record.contains("time") || record.contains("UTC") {
                continue;
            }
            let mut val = record.split(',');
            let time = val.next().unwrap();
            let ecg = val.next().unwrap();
            self.push((time.parse::<u64>()?, ecg.parse::<i32>()?));
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
    fn update_data(&mut self) {
        if let Err(e) = self.update_data_internal() {
            eprintln!("Error getting data: {}", e);
        }
    }
}

impl Chart<Message> for EcgChart {
    type State = ();
    // Create plotters chart
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
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
    pub rr: String,
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

// Instead of reading the output files, get messages containing the data
#[derive(Debug)]
pub struct DataReceiver {
    hr: Receiver<u8>,
    rr: Receiver<String>,
    acc: Receiver<(i16, i16, i16)>,
}

impl DataReceiver {
    pub fn new(hr: Receiver<u8>, rr: Receiver<String>, acc: Receiver<(i16, i16, i16)>) -> Self {
        Self { hr, rr, acc }
    }

    pub fn hr(&self) -> u8 {
        *self.hr.borrow()
    }

    pub fn rr(&self) -> String {
        self.rr.borrow().clone()
    }

    pub fn acc(&self) -> (i16, i16, i16) {
        *self.acc.borrow()
    }
}
