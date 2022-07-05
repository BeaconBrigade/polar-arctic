use super::Message;
use crate::blue::setting::Setting;
use chrono::{DateTime, Utc};
use iced::pure::{
    self, button, column, text_input,
    widget::{PickList, Toggler},
    Pure, State,
};
use iced::{Column, Element, Length, Text};

#[derive(Default)]
pub struct Menu {
    pub meta_state: MetaState,
    pub state: State,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            meta_state: MetaState::default(),
            state: State::new(),
        }
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
            WhichMeta::Id => self.meta_state.meta_data.id = msg,
            WhichMeta::Session => self.meta_state.meta_data.session = msg,
            WhichMeta::Trial => self.meta_state.meta_data.trial = msg,
            WhichMeta::Description => self.meta_state.meta_data.description = msg,
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
#[derive(Debug, Clone)]
pub struct Meta {
    pub id: String,
    pub session: String,
    pub trial: String,
    pub description: String,
    pub date: DateTime<Utc>,
    pub settings: Setting,
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            id: "".to_string(),
            session: "".to_string(),
            trial: "".to_string(),
            description: "".to_string(),
            date: Utc::now(),
            settings: Setting::default(),
        }
    }
}

impl ToString for Meta {
    fn to_string(&self) -> String {
        format!(
            "{},{},{},{},{}\n",
            self.id, self.session, self.trial, self.date, self.description
        )
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

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Hr,
    Acc,
    Ecg,
}

// Store states for meta data
#[derive(Default, Clone)]
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

        let hr_selector = Toggler::new(
            self.meta_data.settings.hr,
            Some("Heart rate".to_string()),
            |b| Message::UpdateSelection(Type::Hr, b),
        );
        let acc_selector = Toggler::new(
            self.meta_data.settings.acc,
            Some("Acceleration".to_string()),
            |b| Message::UpdateSelection(Type::Acc, b),
        );
        let ecg_selector = Toggler::new(
            self.meta_data.settings.ecg,
            Some("Electrocardiagram".to_string()),
            |b| Message::UpdateSelection(Type::Ecg, b),
        );

        let select_title = Text::new("Select range and sample rate (only for acceleration");

        let range_selector = PickList::new(
            vec![2, 4, 8],
            Some(self.meta_data.settings.range),
            Message::RangeChange,
        );
        let rate_selector = PickList::new(
            vec![25, 50, 100, 200],
            Some(self.meta_data.settings.rate),
            Message::RateChange,
        );

        let submit = button(Text::new("Submit")).on_press(Message::NewMeta);

        column()
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(id)
            .push(session)
            .push(trial)
            .push(description)
            .push(hr_selector)
            .push(acc_selector)
            .push(ecg_selector)
            .push(select_title)
            .push(range_selector)
            .push(rate_selector)
            .push(submit)
            .into()
    }
}
