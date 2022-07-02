use crate::menu::Meta;
mod fs;
mod setting;

use setting::Setting;
use fs::init;
use super::modal::arctic;

// manage Bluetooth connections
#[derive(Debug, Default, Clone, Copy)]
pub struct SensorManager {
    settings: Setting,
    event_handler: Handler,
    sensor: Option<()>,
}


impl SensorManager {
    
    pub fn device(&mut self, sensor: ()) {
        self.sensor = Some(sensor)
    }
}

// Create files for storing data
pub async fn update(hr: bool, ecg: bool, acc: bool, metadata: Meta) -> Result<(), tokio::io::Error> {
    let settings = Setting::new(hr, ecg, acc);
    init(settings, metadata).await?;

    Ok(())
}

// Create new device
pub async fn new_device(_id: String) -> Result<(), arctic::Error> {
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Handler;

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler {
    fn new() -> Self {
        Self {}
    }
}

/*impl EventHandler for Handler {

}*/
