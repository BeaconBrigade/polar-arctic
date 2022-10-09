// store what kind of measurements to keep
#[derive(Debug, Clone, Copy)]
pub struct Setting {
    pub hr: bool,
    pub ecg: bool,
    pub acc: bool,
    pub range: u8,
    pub rate: u8,
}

impl Default for Setting {
    fn default() -> Self {
        Self::new(false, false, false, 8, 200)
    }
}

impl Setting {
    pub const fn new(hr: bool, ecg: bool, acc: bool, range: u8, rate: u8) -> Self {
        Self {
            hr,
            ecg,
            acc,
            range,
            rate,
        }
    }
}
