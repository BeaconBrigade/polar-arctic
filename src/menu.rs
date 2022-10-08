use crate::{blue::setting::Setting, modal::PopupMessage, Message};
use chrono::{DateTime, Utc};
use iced::pure::{
    self, button, column, text_input,
    widget::{Column, PickList, Text, Toggler},
    Element,
};
use iced::Length;

#[derive(Default, Debug)]
pub struct Menu {
    pub meta_state: MetaState,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            meta_state: MetaState::default(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let title = Text::new("Metadata").size(30);

        Column::new()
            .push(title)
            .push(self.meta_state.view())
            .into()
    }

    pub fn change_data(&mut self, which: WhichMeta, msg: String) {
        match which {
            WhichMeta::Id => self.meta_state.meta_data.id = msg,
            WhichMeta::Session => self.meta_state.meta_data.session = msg,
            WhichMeta::Trial => self.meta_state.meta_data.trial = msg,
            WhichMeta::Description => self.meta_state.meta_data.description = msg,
            _ => {}
        }
    }

    pub fn verify(&mut self) -> Result<(), WhichMeta> {
        let meta = &mut self.meta_state;

        if meta.meta_data.id.is_empty() {
            return Err(WhichMeta::Id);
        }
        if meta.meta_data.session.is_empty() {
            return Err(WhichMeta::Session);
        }
        if meta.meta_data.trial.is_empty() {
            return Err(WhichMeta::Trial);
        }
        if meta.meta_data.description.is_empty() {
            return Err(WhichMeta::Description);
        }

        if !(meta.meta_data.settings.acc
            || meta.meta_data.settings.ecg
            || meta.meta_data.settings.hr)
        {
            return Err(WhichMeta::NoData);
        }
        if meta.meta_data.settings.hr && meta.paths.hr.is_empty() {
            return Err(WhichMeta::NoPath);
        }
        if meta.meta_data.settings.acc && meta.paths.acc.is_empty() {
            return Err(WhichMeta::NoPath);
        }
        if meta.meta_data.settings.ecg && meta.paths.ecg.is_empty() {
            return Err(WhichMeta::NoPath);
        }

        // get rid of commas to not mess up csv file
        meta.meta_data.id = meta.meta_data.id.replace(',', "-");
        meta.meta_data.session = meta.meta_data.session.replace(',', "-");
        meta.meta_data.trial = meta.meta_data.trial.replace(',', "-");
        meta.meta_data.description = meta.meta_data.description.replace(',', "-");

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
    NoData,
    NoPath,
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Hr,
    Acc,
    Ecg,
}

#[derive(Debug, Default, Clone)]
pub struct Paths {
    pub hr: String,
    pub acc: String,
    pub ecg: String,
}

// Store states for meta data
#[derive(Default, Debug, Clone)]
pub struct MetaState {
    pub meta_data: Meta,
    pub paths: Paths,
}

impl MetaState {
    fn view(&self) -> pure::Element<Message> {
        let help =
            button(Text::new("Help").size(20)).on_press(Message::Popup(PopupMessage::MenuHelp));
        // Meta data inputs
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

        // Toggles for measurement types
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

        // Range and rate selector
        let select_title =
            Text::new("Select range and sample rate (only for acceleration)").size(30);
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

        // Path selectors
        let hr_path = text_input("Path to hr output file", &self.paths.hr, |s| {
            Message::SetPath(Type::Hr, s)
        });
        let acc_path = text_input("Path to acceleration output file", &self.paths.acc, |s| {
            Message::SetPath(Type::Acc, s)
        });
        let ecg_path = text_input(
            "Path to electrocardiagram output file",
            &self.paths.ecg,
            |s| Message::SetPath(Type::Ecg, s),
        );

        let submit = button(Text::new("Submit")).on_press(Message::NewMeta);

        column()
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(help)
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
            .push(hr_path)
            .push(acc_path)
            .push(ecg_path)
            .push(submit)
            .into()
    }
}
