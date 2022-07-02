// store what kind of measurements to keep
#[derive(Debug, Default, Clone, Copy)]
pub struct Setting {
    hr: bool,
    ecg: bool,
    acc: bool,
}

impl Setting {
    pub fn new(hr: bool, ecg: bool, acc: bool) -> Self {
        Self { hr, ecg, acc }
    }

    pub fn get_settings(&self) -> (bool, bool, bool) {
        (self.hr, self.ecg, self.acc)
    }
}
