pub mod fs;
pub mod setting;

use crate::menu::{Meta, Paths};
use arctic::{
    async_trait, Error, EventHandler, H10MeasurementType, HeartRate, NotifyStream, PmdRead,
    PolarSensor,
};
use fs::{init, write_data, write_hr};
use setting::Setting;
use std::sync::Arc;
use tokio::sync::{watch::Receiver, Mutex};

// manage Bluetooth connections
#[derive(Default)]
pub struct SensorManager {
    pub sensor: Option<PolarSensor>,
}

impl SensorManager {
    pub async fn start(&mut self) -> Result<(), Error> {
        if let Some(sensor) = &mut self.sensor {
            sensor.event_loop().await?;
            Ok(())
        } else {
            Err(Error::NoDevice)
        }
    }
}

// Create files for storing data
pub async fn update(
    settings: Setting,
    metadata: Meta,
    paths: Paths,
) -> Result<(), tokio::io::Error> {
    init(settings, metadata, paths).await?;
    Ok(())
}

// Create new device
pub async fn new_device(
    id: String,
    Setting {
        hr,
        ecg,
        acc,
        range,
        rate,
    }: Setting,
    rx: Receiver<bool>,
    paths: Paths,
) -> Result<PolarSensor, Error> {
    let mut sensor = PolarSensor::new(id).await?;

    while !sensor.is_connected().await {
        match sensor.connect().await {
            Err(Error::NoBleAdaptor) => {
                eprintln!("No bluetooth adapter found");
                return Err(Error::NoBleAdaptor);
            }
            Err(why) => eprintln!("Could not connect: {:?}", why),
            _ => {}
        }
    }

    let _ = sensor.range(range);
    let _ = sensor.sample_rate(rate);

    if hr {
        sensor.subscribe(NotifyStream::HeartRate).await?;
    }
    if ecg || acc {
        sensor.subscribe(NotifyStream::MeasurementData).await?;
    }

    if ecg {
        sensor.data_type_push(H10MeasurementType::Ecg)
    }
    if acc {
        sensor.data_type_push(H10MeasurementType::Acc);
    }

    sensor.event_handler(Handler::new(rx, rate, paths));

    Ok(sensor)
}

// Reset sensor for future use
pub async fn reset(manager: Arc<Mutex<SensorManager>>) -> Result<(), Error> {
    let mut unlocked = manager.lock().await;
    let sensor = unlocked.sensor.as_mut().ok_or(Error::NoDevice)?;

    let _ = sensor.range(8);
    let _ = sensor.sample_rate(200);

    let tys = sensor.data_type().as_ref().expect("Unreachable").clone();
    for t in tys {
        sensor.data_type_pop(t);
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct Handler {
    rx: Receiver<bool>,
    rate: u8,
    paths: Paths,
}

impl Handler {
    fn new(rx: Receiver<bool>, rate: u8, paths: Paths) -> Self {
        Self { rx, rate, paths }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn heart_rate_update(&self, _ctx: &PolarSensor, heartrate: HeartRate) {
        if let Err(e) = write_hr(heartrate, &self.paths.hr).await {
            eprintln!("HR writing error: {:?}", e);
        }
    }

    async fn measurement_update(&self, _ctx: &PolarSensor, data: PmdRead) {
        if let Err(e) = write_data(data, self.rate, &self.paths).await {
            eprintln!("measurement writing error: {:?}", e);
        }
    }

    async fn should_continue(&self) -> bool {
        *self.rx.borrow()
    }
}
