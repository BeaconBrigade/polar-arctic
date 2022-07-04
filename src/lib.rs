use iced::{
    self, alignment, executor,
    pure::{Pure, State},
    Application, Column, Command, Container, Element, Length, Rule, Subscription, Text,
};
use iced_aw::{pure::Card, Modal};
use std::sync::Arc;
use std::time;
use tokio::sync::Mutex;

mod blue;
mod data;
mod menu;
mod modal;

use blue::{new_device, setting::Setting, update, SensorManager};
use data::Data;
use menu::{Menu, Type, WhichMeta};
use modal::{get_modal, PopupMessage};

// Main Application
#[derive(Default)]
pub struct App {
    sensor_manager: Arc<Mutex<SensorManager>>,
    view: Views,
    which_err: PopupMessage,
    modal_state: iced_aw::modal::State<State>,
    running: bool,
}

// Possible views to show the user
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
    CloseModal,
    Popup(PopupMessage),
    Connected,
    UpdateSelection(Type, bool),
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
                if !self.running {
                    if let Views::Data(data) = &mut self.view {
                        let other_me = Arc::clone(&self.sensor_manager);
                        Command::perform(
                            new_device(data.id().clone(), Setting::new(true, true, true)), // FIX
                            move |res| match res {
                                Ok(sensor) => {
                                    futures::executor::block_on(other_me.lock()).sensor =
                                        Some(sensor);
                                    Message::Popup(PopupMessage::Connected)
                                }
                                Err(e) => Message::Popup(PopupMessage::Polar(e.to_string())),
                            },
                        )
                    } else {
                        Command::none()
                    }
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
                        let set = meta.meta_state.meta_data.settings;
                        self.update(Message::SwitchView(WhichView::Data));
                        return Command::perform(update(set, data), |res| {
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
                Command::none()
            }
            Message::CloseModal => {
                self.modal_state.show(false);
                Command::none()
            }
            Message::Popup(which) => {
                self.modal_state.show(true);
                self.which_err = which;
                Command::none()
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
                        Type::Hr => menu.meta_state.meta_data.settings.hr = b,
                        Type::Acc => menu.meta_state.meta_data.settings.acc = b,
                        Type::Ecg => menu.meta_state.meta_data.settings.ecg = b,
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

    fn view(&mut self) -> Element<'_, Message> {
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

        Modal::new(&mut self.modal_state, content, |state| {
            let (title, body) = get_modal(self.which_err.clone());
            let body = iced::pure::widget::Text::new(body);

            let card = Card::new(iced::pure::widget::Text::new(title), body)
                .max_width(300)
                .on_close(Message::CloseModal);

            Pure::new(state, card).into()
        })
        .backdrop(Message::CloseModal)
        .on_esc(Message::CloseModal)
        .into()
    }
}
