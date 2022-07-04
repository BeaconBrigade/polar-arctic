// store what kind of measurements to keep
#[derive(Debug, Default, Clone, Copy)]
pub struct Setting {
    pub hr: bool,
    pub ecg: bool,
    pub acc: bool,
}

impl Setting {
    pub fn new(hr: bool, ecg: bool, acc: bool) -> Self {
        Self { hr, ecg, acc }
    }
}
