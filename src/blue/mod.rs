mod fs;
pub mod setting;

use crate::menu::Meta;
use arctic::{async_trait, Error, EventHandler, H10MeasurementType, NotifyStream, PolarSensor};
use fs::init;
use setting::Setting;

// manage Bluetooth connections
#[derive(Default)]
pub struct SensorManager {
    pub sensor: Option<PolarSensor>,
}

impl SensorManager {
    pub async fn start(&mut self) -> Result<(), Error> {
        if let Some(sensor) = &mut self.sensor {
            sensor.event_handler(Handler::new());
            sensor.event_loop().await?;
            Ok(())
        } else {
            Err(Error::NoDevice)
        }
    }
}

// Create files for storing data
pub async fn update(settings: Setting, metadata: Meta) -> Result<(), tokio::io::Error> {
    init(settings, metadata).await?;
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

    Ok(sensor)
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

#[async_trait]
impl EventHandler for Handler {}
