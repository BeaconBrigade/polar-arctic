use iced::pure::{button, column, row, text_input, widget::Text, Pure, State};
use iced::{Column, Length, Row, Rule};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend};
use rev_lines::RevLines;
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use tokio::sync::watch::Receiver;

use super::{modal::PopupMessage, Message, WhichView};

pub struct Data {
    chart: EcgChart,
    device_id: String,
    state: State,
    recent_data: Recent,
    receiver: Option<DataReceiver>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            chart: EcgChart::new().unwrap(),
            device_id: "".to_string(),
            state: State::new(),
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

    pub fn view(&mut self) -> iced::Element<Message> {
        let back = button(Text::new("Back to menu").size(20))
            .on_press(Message::SwitchView(WhichView::Menu))
            .padding(15);
        let help = button(Text::new("Help").size(20))
            .on_press(Message::Popup(PopupMessage::DataHelp))
            .padding(15);

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

        let rr_text = &self.recent_data.rr;
        let mut rr_text = rr_text.chars();
        rr_text.next();

        let bpm = iced::Text::new(&format!("Heart rate (BPM): {}", self.recent_data.bpm));
        let rr = iced::Text::new(&format!(
            "RR interval (µV): {}",
            rr_text.as_str().replace(',', ", ")
        ));
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

        Column::new().spacing(20).push(pure).push(data).into()
    }

    pub fn update_id(&mut self, msg: String) {
        self.device_id = msg;
    }

    pub fn id(&mut self) -> &mut String {
        &mut self.device_id
    }

    pub fn update(&mut self) {
        self.chart.update();
        if let Some(rx) = &self.receiver {
            self.recent_data.bpm = rx.hr();
            self.recent_data.rr = rx.rr();
            let (x, y, z) = rx.acc();
            self.recent_data.x = x;
            self.recent_data.y = y;
            self.recent_data.z = z;
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.chart.path = Some(path)
    }
}

// Store chart data
#[derive(Default)]
struct EcgChart {
    data_points: VecDeque<(u64, i32)>,
    pub path: Option<String>,
}

impl EcgChart {
    pub fn new() -> Result<EcgChart, Box<dyn std::error::Error>> {
        let mut chart = Self {
            data_points: VecDeque::with_capacity(200),
            path: None,
        };
        chart.update_data()?;

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
    fn update_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
    fn update(&mut self) {
        if let Err(e) = self.update_data() {
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
    pub rr: String,
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

// Instead of reading the output files, get messages containing the data
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
