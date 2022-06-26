use iced::pure::{
    text_input, button, widget::Text, column, row, Pure, State,
};
use iced::{
    Length, Rule, Column,
};
use csv::ReaderBuilder;
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
        Self { chart: EcgChart::new().unwrap(), ..Default::default() }
    }

    pub fn view(&mut self) -> iced::Element<Message> {
        let back = button(
            Text::new("Back to menu").size(20),
        ).on_press(Message::SwitchView(WhichView::Menu));

        let header = row()
            .push(Text::new("Data"))
            .push(Rule::horizontal(0))
            .push(back);

        let input = text_input(
            "Device ID",
            &self.device_id,
            Message::NewDeviceID,
        )
        .padding(15)
        .size(20)
        .on_submit(Message::CreateSensor);

        // Data side
        let view = column()
            .spacing(20)
            .width(Length::Fill)
            .max_width(1000)
            .push(header)
            .push(Rule::horizontal(10))
            .push(input);

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

    pub fn update(&mut self) {
        self.chart.update();
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
