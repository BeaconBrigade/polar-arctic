pub mod fs;
pub mod setting;

use crate::{
    data::DataReceiver,
    menu::{Meta, Paths},
};
use arctic::{
    async_trait, Error, EventHandler, H10MeasurementType, HeartRate, NotifyStream, PmdRead,
    PolarSensor,
};
use fs::{init, write_data, write_hr};
use setting::Setting;
use std::sync::{self, Arc};
use tokio::sync::{
    watch::{channel, Receiver, Sender},
    Mutex,
};

// manage Bluetooth connections
#[derive(Default)]
pub struct SensorManager {
    pub sensor: Option<PolarSensor>,
}

impl std::fmt::Debug for SensorManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SensorManager")
            .field("sensor", &"PolarSensor { /* private fields */ }")
            .finish()
    }
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
    sender: DataSender,
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

    sensor.event_handler(Handler::new(rx, rate, paths, sender));

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

// Handle bluetooth events
struct Handler {
    rx: Receiver<bool>,
    rate: u8,
    paths: Paths,
    sender: DataSender,
    hr_start: sync::Mutex<Option<u64>>,
    pmd_start: sync::Mutex<Option<u64>>,
}

impl Handler {
    fn new(rx: Receiver<bool>, rate: u8, paths: Paths, sender: DataSender) -> Self {
        Self {
            rx,
            rate,
            paths,
            sender,
            hr_start: sync::Mutex::new(None),
            pmd_start: sync::Mutex::new(None),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn heart_rate_update(&self, _ctx: &PolarSensor, heartrate: HeartRate) {
        match write_hr(heartrate, &self.paths.hr, &self.hr_start).await {
            Ok(last) => {
                self.sender.hr(last.0);
                self.sender.rr(last.1);
            }
            Err(e) => eprintln!("HR writing error: {:?}", e),
        }
    }

    async fn measurement_update(&self, _ctx: &PolarSensor, data: PmdRead) {
        match write_data(data, self.rate, &self.paths, &self.pmd_start).await {
            Ok(Some(last)) => {
                self.sender.acc(last);
            }
            Err(e) => eprintln!("Measurement writing error: {:?}", e),
            _ => {}
        }
    }

    async fn should_continue(&self) -> bool {
        *self.rx.borrow()
    }
}

pub struct DataSender {
    hr: Sender<u8>,
    rr: Sender<String>,
    acc: Sender<(i16, i16, i16)>,
}

impl DataSender {
    pub fn init_transmitters() -> (Self, DataReceiver) {
        let (hr_tx, hr_rx) = channel(0);
        let (rr_tx, rr_rx) = channel("".to_string());
        let (acc_tx, acc_rx) = channel((0, 0, 0));

        (
            Self {
                hr: hr_tx,
                rr: rr_tx,
                acc: acc_tx,
            },
            DataReceiver::new(hr_rx, rr_rx, acc_rx),
        )
    }

    pub fn hr(&self, hr: u8) {
        self.hr.send(hr).expect("hr sender failed");
    }

    pub fn rr(&self, rr: String) {
        self.rr.send(rr).expect("ecg sender failed");
    }

    pub fn acc(&self, acc: (i16, i16, i16)) {
        self.acc.send(acc).expect("acc sender failed");
    }
}
