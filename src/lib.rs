use iced::{
    self, alignment, executor, Command, Length,
    Subscription, Text, pure::{Application, Element, widget::{Column, Container, Rule}}
};
use iced_aw::pure::{Card, Modal};
use std::sync::Arc;
use std::time;
use tokio::sync::{
    watch::{channel, Sender},
    Mutex,
};

mod blue;
mod data;
mod menu;
mod modal;

use blue::{new_device, reset, setting::Setting, update, DataSender, SensorManager};
use data::Data;
use menu::{Menu, Paths, Type, WhichMeta};
use modal::{get_modal, PopupMessage};

// Main Application
#[derive(Debug, Default)]
pub struct App {
    sensor_manager: Arc<Mutex<SensorManager>>,
    view: Views,
    which_err: PopupMessage,
    show_err: bool,
    settings: Setting,
    tx: Option<Sender<bool>>,
    paths: Paths,
}

// Possible views to show the user
#[derive(Debug)]
pub enum Views {
    Menu(Box<Menu>),
    Data(Box<Data>),
}

impl Views {
    fn view(&self) -> Element<Message> {
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
    CloseModal,
    Popup(PopupMessage),
    Connected,
    UpdateSelection(Type, bool),
    RangeChange(u8),
    RateChange(u8),
    StopMeasurement,
    SetPath(Type, String),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;

    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        "Polar-Arctic".to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => Command::none(),
            Message::Tick => {
                if let Views::Data(data) = &mut self.view {
                    data.update();
                }
                Command::none()
            }
            Message::NewDeviceID(msg) => {
                if let Views::Data(data) = &mut self.view {
                    data.update_id(msg);
                }
                Command::none()
            }
            Message::CreateSensor => {
                // Replace with new using user selected options
                if let Views::Data(data) = &mut self.view {
                    let (tx, rx) = channel(true);
                    self.tx = Some(tx);
                    let set = self.settings;
                    let paths = self.paths.clone();
                    let other_me = Arc::clone(&self.sensor_manager);
                    let (send, recv) = DataSender::init_transmitters();
                    data.take_receivers(recv);
                    Command::perform(
                        new_device(
                            data.id().clone(),
                            Setting::new(set.hr, set.ecg, set.acc, set.range, set.rate),
                            rx,
                            paths,
                            send,
                        ),
                        move |res| match res {
                            Ok(sensor) => {
                                futures::executor::block_on(other_me.lock()).sensor = Some(sensor);
                                Message::Popup(PopupMessage::Connected)
                            }
                            Err(e) => Message::Popup(PopupMessage::Polar(e.to_string())),
                        },
                    )
                } else {
                    Command::none()
                }
            }
            Message::NewMeta => {
                if let Views::Menu(meta) = &mut self.view {
                    if let Err(which) = meta.verify() {
                        self.update(Message::Popup(which.into()));
                    } else {
                        let data = meta.meta_state.meta_data.clone();
                        let set = self.settings;
                        let paths = meta.meta_state.paths.clone();
                        let ecg = paths.ecg.clone();
                        self.update(Message::SwitchView(WhichView::Data));
                        if let Views::Data(data) = &mut self.view {
                            data.set_path(ecg);
                        }
                        return Command::perform(update(set, data, paths), |res| {
                            if let Err(err) = res {
                                Message::Popup(PopupMessage::Io(err.to_string()))
                            } else {
                                Message::None
                            }
                        });
                    }
                }
                Command::none()
            }
            Message::ChangeMeta(which, msg) => {
                if let Views::Menu(meta) = &mut self.view {
                    meta.change_data(which, msg);
                }
                Command::none()
            }
            Message::SwitchView(view) => {
                self.view = view.into();
                if let WhichView::Menu = view {
                    self.update(Message::StopMeasurement);
                    let other_me = Arc::clone(&self.sensor_manager);
                    Command::perform(reset(other_me), |res| {
                        if let Err(e) = res {
                            Message::Popup(PopupMessage::Polar(e.to_string()))
                        } else {
                            Message::None
                        }
                    })
                } else {
                    Command::none()
                }
            }
            Message::CloseModal => {
                self.show_err = false;
                Command::none()
            }
            Message::Popup(which) => {
                self.show_err = true;
                self.which_err = which;
                if let PopupMessage::Connected = &self.which_err {
                    self.update(Message::Connected)
                } else {
                    Command::none()
                }
            }
            Message::Connected => {
                let other_me = Arc::clone(&self.sensor_manager);
                Command::perform(
                    tokio::spawn(async move { other_me.lock().await.start().await }),
                    |res| {
                        if let Err(e) = res {
                            Message::Popup(PopupMessage::Polar(e.to_string()))
                        } else {
                            Message::None
                        }
                    },
                )
            }
            Message::UpdateSelection(t, b) => {
                if let Views::Menu(menu) = &mut self.view {
                    match t {
                        Type::Hr => {
                            self.settings.hr = b;
                            menu.meta_state.meta_data.settings.hr = b;
                        }
                        Type::Acc => {
                            self.settings.acc = b;
                            menu.meta_state.meta_data.settings.acc = b;
                        }
                        Type::Ecg => {
                            self.settings.ecg = b;
                            menu.meta_state.meta_data.settings.ecg = b;
                        }
                    }
                }
                Command::none()
            }
            Message::RangeChange(num) => {
                if let Views::Menu(menu) = &mut self.view {
                    self.settings.range = num;
                    menu.meta_state.meta_data.settings.range = num;
                }
                Command::none()
            }
            Message::RateChange(num) => {
                if let Views::Menu(menu) = &mut self.view {
                    self.settings.rate = num;
                    menu.meta_state.meta_data.settings.rate = num;
                }
                Command::none()
            }
            Message::StopMeasurement => {
                if let Some(tx) = &self.tx {
                    tx.send(false).expect("Unable to send stop signal????");
                }
                Command::none()
            }
            Message::SetPath(ty, path) => {
                if let Views::Menu(menu) = &mut self.view {
                    match ty {
                        Type::Hr => {
                            menu.meta_state.paths.hr = path.clone();
                            self.paths.hr = path;
                        }
                        Type::Ecg => {
                            menu.meta_state.paths.ecg = path.clone();
                            self.paths.ecg = path;
                        }
                        Type::Acc => {
                            menu.meta_state.paths.acc = path.clone();
                            self.paths.acc = path;
                        }
                    }
                }
                Command::none()
            }
        }
    }

    // Tick every 16ms to update graph
    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(time::Duration::from_millis(100)).map(|_| Message::Tick)
    }

    fn view(&self) -> Element<'_, Message> {
        let title = Text::new("Polar-Arctic")
            .width(Length::Fill)
            .size(60)
            .horizontal_alignment(alignment::Horizontal::Center);

        let body = self.view.view();

        let content = Container::new(
            Column::new()
                .push(title)
                .push(Rule::horizontal(10))
                .push(body),
        );

        let (title, body) = get_modal(&self.which_err);

        Modal::new(self.show_err, content, move || {
            Card::new(Text::new(title), Text::new(&body))
                .max_width(300)
                .on_close(Message::CloseModal)
                .into()
        })
            .backdrop(Message::CloseModal)
            .on_esc(Message::CloseModal)
            .into()
    }
}
