use super::menu::WhichMeta;

mod description;
mod device;
mod id;
mod session;
mod trial;

// Decide which card to send
#[derive(Debug, Clone)]
pub enum PopupMessage {
    Meta(WhichMeta),
    DeviceID,
    Polar(String),
    Io(String),
    Connected,
}

impl Default for PopupMessage {
    fn default() -> Self {
        Self::DeviceID
    }
}

impl From<WhichMeta> for PopupMessage {
    fn from(which: WhichMeta) -> Self {
        PopupMessage::Meta(which)
    }
}

pub fn get_modal(ty: PopupMessage) -> (String, String) {
    match ty {
        PopupMessage::Meta(err) => (
            "Form not completed".to_string(),
            match err {
                WhichMeta::Id => id::view(),
                WhichMeta::Trial => trial::view(),
                WhichMeta::Session => session::view(),
                WhichMeta::Description => description::view(),
            },
        ),
        PopupMessage::DeviceID => ("Invalid device ID".to_string(), device::view()),
        PopupMessage::Polar(err) => ("Bluetooth error".to_string(), err),
        PopupMessage::Io(err) => ("Error finding output file".to_string(), err),
        PopupMessage::Connected => (
            "Device connected!".to_string(),
            "Device connected!".to_string(),
        ),
    }
}
