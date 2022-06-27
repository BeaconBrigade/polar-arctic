use super::Message;
use chrono::{DateTime, Utc};
use iced::pure::{self, button, column, text_input, Pure, State};
use iced::{Column, Element, Length, Text};

#[derive(Default)]
pub struct Menu {
    meta_state: MetaState,
    state: State,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            meta_state: MetaState::default(),
            state: State::new(),
        }
    }

    pub fn state(&mut self) -> &mut MetaState {
        &mut self.meta_state
    }

    pub fn view(&mut self) -> Element<Message> {
        let title = Text::new("Metadata").size(30);

        Column::new()
            .push(title)
            .push(Pure::new(&mut self.state, self.meta_state.view()))
            .into()
    }

    pub fn change_data(&mut self, which: WhichMeta, msg: String) {
        match which {
            WhichMeta::Id => self.state().meta_data.id = msg,
            WhichMeta::Session => self.state().meta_data.session = msg,
            WhichMeta::Trial => self.state().meta_data.trial = msg,
            WhichMeta::Description => self.state().meta_data.description = msg,
        }
    }

    pub fn verify(&mut self) -> Result<(), WhichMeta> {
        let meta = &mut self.meta_state.meta_data;

        if meta.id.is_empty() {
            return Err(WhichMeta::Id);
        }
        if meta.session.is_empty() {
            return Err(WhichMeta::Session);
        }
        if meta.trial.is_empty() {
            return Err(WhichMeta::Trial);
        }
        if meta.description.is_empty() {
            return Err(WhichMeta::Description);
        }

        Ok(())
    }
}

// Store meta-data about this run
pub struct Meta {
    pub id: String,
    pub session: String,
    pub trial: String,
    pub description: String,
    pub date: DateTime<Utc>,
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            id: "".to_string(),
            session: "".to_string(),
            trial: "".to_string(),
            description: "".to_string(),
            date: Utc::now(),
        }
    }
}

// Which kind of metadata to change
#[derive(Debug, Clone, Copy)]
pub enum WhichMeta {
    Id,
    Session,
    Trial,
    Description,
}

// Store states for meta data
#[derive(Default)]
pub struct MetaState {
    pub meta_data: Meta,
}

impl MetaState {
    fn view(&mut self) -> pure::Element<Message> {
        let id = text_input("Participant ID", &self.meta_data.id, |s| {
            Message::ChangeMeta(WhichMeta::Id, s)
        });

        let session = text_input("Session Number", &self.meta_data.session, |s| {
            Message::ChangeMeta(WhichMeta::Session, s)
        });

        let trial = text_input("Trial number", &self.meta_data.trial, |s| {
            Message::ChangeMeta(WhichMeta::Trial, s)
        });

        let description = text_input("Description/Notes", &self.meta_data.description, |s| {
            Message::ChangeMeta(WhichMeta::Description, s)
        });

        let submit = button(Text::new("Submit")).on_press(Message::NewMeta);

        column()
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(id)
            .push(session)
            .push(trial)
            .push(description)
            .push(submit)
            .into()
    }
}
