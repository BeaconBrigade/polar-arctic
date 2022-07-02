use iced::{
    self, Application, Column, Length, Subscription, Rule, alignment, executor, 
    Command, Element, Text, Container, pure::{Pure, State},
};
use iced_aw::{pure::Card, Modal};
use std::time;

mod blue;
mod data;
mod menu;
mod modal;

use data::Data;
use menu::{Menu, WhichMeta};
use modal::{get_body, PopupMessage, arctic};
use blue::{SensorManager, new_device, update};


// Main Application
#[derive(Default)]
pub struct App {
    sensor_manager: SensorManager,
    view: Views,
    which_err: PopupMessage,
    modal_state: iced_aw::modal::State<State>,
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
    SetSensor(()),
    NewMeta,
    ChangeMeta(WhichMeta, String),
    SwitchView(WhichView),
    CloseModal,
    Popup(PopupMessage),
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
            Message::None => {
                Command::none()
            }
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
                    Command::perform(
                        new_device(data.id().clone()),
                        |res| {
                            if let Ok(sensor) = res {
                                Message::SetSensor(sensor)
                            } else {
                                Message::Popup(PopupMessage::Polar(arctic::Error::Dumb))
                            }
                        }
                    )
                } else {
                    Command::none()
                }
            }
            Message::SetSensor(sensor) => {
                self.sensor_manager.device(sensor);
                Command::none()
            }
            Message::NewMeta => {
                if let Views::Menu(meta) = &mut self.view {
                    if let Err(which) = meta.verify() {
                        self.update(Message::Popup(which.into()));
                    } else {
                        let data = meta.state().meta_data.clone();
                        self.update(Message::SwitchView(WhichView::Data));
                        return Command::perform(
                            update(true, true, true, data), // FIX LATER
                            |res| {
                                if res.is_err() {
                                    Message::Popup(PopupMessage::Io)
                                } else {
                                    Message::None
                                }
                            }
                        );
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
        }
    }

    // Tick every 16ms to update graph
    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(time::Duration::from_millis(16)).map(|_| Message::Tick)
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
                .push(body)
        );

        Modal::new(&mut self.modal_state, content,
            |state| {
                let body = iced::pure::widget::Text::new(get_body(self.which_err));

                let card = Card::new(
                    iced::pure::widget::Text::new("Error Occured"),
                    body,
                )
                .max_width(300)
                .on_close(Message::CloseModal);

                Pure::new(state, card)
                    .into()
            }
        )
        .backdrop(Message::CloseModal)
        .on_esc(Message::CloseModal)
        .into()
    }
}
