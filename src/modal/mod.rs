use super::menu::WhichMeta;

mod id;
mod trial;
mod session;
mod description;
mod device;
mod ble;

// Decide which card to send
#[derive(Debug, Clone, Copy)]
pub enum PopupMessage {
    Meta(WhichMeta),
    DeviceID,
    Polar(arctic::Error),
    Io,
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

pub fn get_body(ty: PopupMessage) -> String {
    match ty {
        PopupMessage::Meta(err) => {
            match err {
                WhichMeta::Id => id::view(),
                WhichMeta::Trial => trial::view(),
                WhichMeta::Session => session::view(),
                WhichMeta::Description => description::view(),
            }
        }
        PopupMessage::DeviceID => {
            device::view()
        }
        PopupMessage::Polar(err) => {
            match err {
                arctic::Error::Dumb => ble::view(err),
                arctic::Error::Stupid => ble::view(err),
            }
        }
        PopupMessage::Io => {
            "IO error, output directory not found.".to_string()
        }
    }
}

pub mod arctic {
    #[derive(Debug, Clone, Copy)]
    pub enum Error {
        Dumb,
        Stupid,
    }
}
