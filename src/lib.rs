use iced::{
    self, Application, Column, Length, Subscription, Rule, alignment, executor, 
    Command, Element, Text,
};
use std::time;

mod blue;
mod menu;
mod data;

use menu::{Menu, WhichMeta};
use data::Data;

#[derive(Default)]
pub struct App {
    sensor: Option<()>, // TODO - fill with actual arctic sensor when update is made
    view: Views,
}

pub enum Views {
    Menu(Box<Menu>),
    Data(Box<Data>),
}

impl Views {
    fn view(&mut self) -> iced::Element<Message> {
        match self {
            Views::Menu(menu) => menu.view(),
            Views::Data(data) => data.view(),
        }
    }
}

impl Default for Views {
    fn default() -> Self {
        Views::Menu(Box::new(Menu::new()))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WhichView {
    Menu,
    Data,
}

impl From<WhichView> for Views {
    fn from(which: WhichView) -> Self {
        match which {
            WhichView::Menu => Views::Menu(Box::new(Menu::default())),
            WhichView::Data => Views::Data(Box::new(Data::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Tick,
    NewDeviceID(String),
    CreateSensor,
    NewMeta,
    ChangeMeta(WhichMeta, String),
    SwitchView(WhichView),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            App::default(),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Polar-Arctic".to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => {}
            Message::Tick => {
                if let Views::Data(data) = &mut self.view {
                    data.update();
                }
            }
            Message::NewDeviceID(msg) => {
                if let Views::Data(data) = &mut self.view {
                    data.update_id(msg);
                }
            }
            Message::CreateSensor => {
                // Construct sensor
                println!("Sensor created!");
            }
            Message::NewMeta => {
                if let Views::Menu(meta) = &mut self.view {
                    if meta.verify().is_ok() {
                        self.update(Message::SwitchView(WhichView::Data));
                    }
                }
                // TODO - Update output file here
            }
            Message::ChangeMeta(which, msg) => {
                if let Views::Menu(meta) = &mut self.view {
                    meta.change_data(which, msg);
                }
            }
            Message::SwitchView(view) => {
                self.view = view.into();
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

        let body = self.view.view();

        Column::new()
            .push(title)
            .push(Rule::horizontal(10))
            .push(body)
            .into()
    }
}

