use csv::ReaderBuilder;
use iced::pure::{button, column, row, text_input, widget::Text, Pure, State};
use iced::{Column, Length, Rule};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, DrawingBackend};
use std::collections::VecDeque;

use super::{Message, WhichView};

pub struct Data {
    chart: EcgChart,
    device_id: String,
    state: State,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            chart: EcgChart::new().unwrap(),
            device_id: "".to_string(),
            state: State::new(),
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Self {
            chart: EcgChart::new().unwrap(),
            ..Default::default()
        }
    }

    pub fn view(&mut self) -> iced::Element<Message> {
        let back = button(Text::new("Back to menu").size(20))
            .on_press(Message::SwitchView(WhichView::Menu));

        let header = row()
            .push(Text::new("Data"))
            .push(Rule::horizontal(0))
            .push(back);

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

        Column::new()
            .spacing(20)
            .push(pure)
            .push(self.chart.view())
            .into()
    }

    pub fn update_id(&mut self, msg: String) {
        self.device_id = msg;
    }

    pub fn id(&mut self) -> &mut String {
        &mut self.device_id
    }

    pub fn update(&mut self) {
        self.chart.update();
    }
}

// Store chart data
#[derive(Default)]
struct EcgChart {
    data_points: VecDeque<(u64, i32)>,
}


impl EcgChart {
    pub fn new() -> Result<EcgChart, Box<dyn std::error::Error>> {
        let mut chart = Self::default();
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
            .from_path("output/test.csv")
            .unwrap();

        let records = rdr.records().skip(1).collect::<Vec<Result<csv::StringRecord, csv::Error>>>().into_iter().rev().take(200).rev();

        // skip extra header row
        for record in records {
            let result = record?;
            self.push((result[0].parse()?, result[1].parse::<i32>()?));
        }
        Ok(())
    }

    // Add to back and pop off front
    fn push(&mut self, val: (u64, i32)) {
        self.data_points.push_back(val);
        while self.data_points.len() > 200 {
            self.data_points.pop_front();
        }
    }

    // Update data - remove data from the end and add to the start
    fn update(&mut self) {}
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

        ctx.draw_series(series)
            .expect("Error making graph");
    }
}
