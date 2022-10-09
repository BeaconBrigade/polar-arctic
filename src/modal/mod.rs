use super::menu::WhichMeta;

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
        Self::Meta(which)
    }
}

pub fn get_modal(ty: &PopupMessage) -> (&'static str, String) {
    match ty {
        PopupMessage::Meta(err) => (
            "Form not completed",
            match err {
                WhichMeta::Id => "A participant ID must be specified".to_owned(),
                WhichMeta::Trial => "A trial must be specified".to_owned(),
                WhichMeta::Session => "A session must be specified".to_owned(),
                WhichMeta::Description => "A description must be provided".to_owned(),
                WhichMeta::NoData => "At least one measurement type must be specified".to_owned(),
                WhichMeta::NoPath => "A file path must be specified for each selected measurement type".to_owned(),
            },
        ),
        PopupMessage::DeviceID => ("Invalid device ID", "Invalid device ID. Device IDs are 6 characters long".to_owned()),
        PopupMessage::Polar(err) => ("Bluetooth error", err.clone()),
        PopupMessage::Io(err) => ("Error finding output file", err.clone()),
        PopupMessage::Connected => (
            "Device connected!",
            "Device connected!".to_owned(),
        ),
        PopupMessage::MenuHelp => ("Help", "The first four boxes are for filling in data regarding your session. Each of these boxes must be filled in. The three toggles following allow you to select which measurement types you want collected. You must select at least one data type. The last three text boxes allow you to choose where you would like your data saved. For every data type you select measurement for, you must specify a file path for it to write to. Each file path is interpreted relatively (`/` or `~` don't work). Click submit when you're done entering your data.".to_owned()),
        PopupMessage::DataHelp => ("Help", "The `Device ID` box is where you type in your polar sensor's device ID. Press enter to start connecting to the device. A popup will appear to tell if you connection was successful or if it failed. The `Back to Menu` button will return you to the starting screen. Press `Stop Measurement` to stop collecting data from the sensor. The graph and other text display your sensor's data.".to_owned()),
    }
}
