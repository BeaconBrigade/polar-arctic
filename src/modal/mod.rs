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
    MenuHelp,
    DataHelp,
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
                WhichMeta::NoData => "At least one measurement type must be specified".to_string(),
                WhichMeta::NoPath => "A file path must be specified for each selected measurement type".to_string(),
            },
        ),
        PopupMessage::DeviceID => ("Invalid device ID".to_string(), device::view()),
        PopupMessage::Polar(err) => ("Bluetooth error".to_string(), err),
        PopupMessage::Io(err) => ("Error finding output file".to_string(), err),
        PopupMessage::Connected => (
            "Device connected!".to_string(),
            "Device connected!".to_string(),
        ),
        PopupMessage::MenuHelp => ("Help".to_string(), "The first four boxes are for filling in data regarding your session. Each of these boxes must be filled in. The three toggles following allow you to select which measurement types you want collected. You must select at least one data type. The last three text boxes allow you to choose where you would like your data saved. For every data type you select measurement for, you must specify a file path for it to write to. Each file path is interpreted relatively (`/` or `~` don't work). Click submit when you're done entering your data.".to_string()),
        PopupMessage::DataHelp => ("Help".to_string(), "The `Device ID` box is where you type in your polar sensor's device ID. Press enter to start connecting to the device. A popup will appear to tell if you connection was successful or if it failed. The `Back to Menu` button will return you to the starting screen. Press `Stop Measurement` to stop collecting data from the sensor. The graph and other text display your sensor's data.".to_string()),
    }
}
