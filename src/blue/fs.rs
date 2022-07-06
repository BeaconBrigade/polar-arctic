use super::setting::Setting;
use crate::menu::Meta;
use arctic::{PmdRead, H10MeasurementType, PmdData, HeartRate};
use tokio::{
    fs::OpenOptions,
    io::{AsyncWriteExt, BufWriter, Error},
};
use std::time::{UNIX_EPOCH, SystemTime};

enum MeasureType {
    Hr,
    Ecg,
    Acc,
}

impl ToString for MeasureType {
    fn to_string(&self) -> String {
        match self {
            MeasureType::Hr => "time,bpm,rr\n",
            MeasureType::Ecg => "time,val\n",
            MeasureType::Acc => "time,x,y,z\n",
        }
        .to_string()
    }
}

// Create/Truncate all data
pub async fn init(Setting { hr, ecg, acc, .. }: Setting, metadata: Meta) -> Result<(), Error> {
    if hr {
        add_headers(MeasureType::Hr, "output/hr.csv", metadata.to_string()).await?;
    }

    if ecg {
        add_headers(MeasureType::Ecg, "output/ecg.csv", metadata.to_string()).await?;
    }

    if acc {
        add_headers(MeasureType::Acc, "output/acc.csv", metadata.to_string()).await?;
    }

    Ok(())
}

// Add headers to each csv file
async fn add_headers(ty: MeasureType, path: &str, mut msg: String) -> Result<(), Error> {
    let output = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .await?;
    let mut writer = BufWriter::with_capacity(200, output);
    msg.push_str(&ty.to_string());

    writer.write_all(msg.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

// Write ecg/acc data to file
pub async fn write_data(data: PmdRead, rate: u8) -> Result<(), Error> {
    let outpath = match data.data_type() {
        H10MeasurementType::Acc => "output/acc.csv",
        H10MeasurementType::Ecg => "output/ecg.csv",
    };

    let outfile = OpenOptions::new()
        .append(true)
        .open(outpath)
        .await?;

    let mut writer = BufWriter::with_capacity(400, outfile);
    let msg = generate_msg(data, rate);

    writer.write_all(msg.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

// Create msg to write to csv file
fn generate_msg(data: PmdRead, rate: u8) -> String {
    let mut msg = "".to_string();
    let mut timestamp = data.time_stamp();

    // change in timestamp between samples
    let offset = (1.0 / (match data.data_type() {
        H10MeasurementType::Acc => rate,
        H10MeasurementType::Ecg => 130,
    } as f64 * 1.0e-9)) as u64; // convert hz to ns

    for d in data.data() {
        match d {
            PmdData::Acc(acc) => {
                let (x, y, z) = acc.data();
                msg.push_str(format!("{},{},{},{}\n", timestamp, x, y, z).as_str());
            }
            PmdData::Ecg(ecg) => {
                msg.push_str(format!("{},{}\n", timestamp, ecg.val()).as_str());
            }
        }
        timestamp -= offset;
    }

    msg
}

const DIFF_FROM_H10_TO_UNIX: u64 = 946684800000000000;

// Write hr data
pub async fn write_hr(data: HeartRate) -> Result<(), Error> {
    let outfile = OpenOptions::new()
        .append(true)
        .open("output/hr.csv")
        .await?;

    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards????????");

    let timestamp = unix.as_nanos() - DIFF_FROM_H10_TO_UNIX as u128;

    let mut rr = "".to_string();
    let stupid = vec![]; // unwanted silly empty array
    let rr_data = data.rr().as_ref().unwrap_or(&stupid);
    for i in rr_data {
        rr.push_str(format!(",{}", i).as_str());
    }

    let mut writer = BufWriter::with_capacity(200, outfile);
    let msg = format!("{},{}{}\n", timestamp, data.bpm(), rr);

    writer.write_all(msg.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_get_msg_ecg() {
        let msg = generate_msg(PmdRead::new(vec![
            0x00, 0xea, 0x54, 0xa2, 0x42, 0x8b, 0x45, 0x52, 0x08, 0x00, 0xff, 0xff, 0xff, 0x00, 0x00, 0x10,
        ])
        .unwrap(), 200);
        let timestamp = 599618164814402794u64;
        let new_time = timestamp - 7692307;

        assert!(msg.contains(&format!("{}", timestamp)));
        assert!(msg.contains(&format!("{}", new_time)));
    }

    #[test]
    fn try_get_msg_acc() {
        let msg = generate_msg(PmdRead::new(vec![
            0x02, 0xea, 0x54, 0xa2, 0x42, 0x8b, 0x45, 0x52, 0x08, 0x01, 0x45, 0xff, 0xe4, 0xff,
            0xb5, 0x03, 0x45, 0xff, 0xe4, 0xff, 0xb8, 0x03,
        ])
        .unwrap(), 200);

        let timestamp = 599618164814402794u64;
        let new_time = timestamp - 5000000;

        assert!(msg.contains(&format!("{}", timestamp)));
        assert!(msg.contains(&format!("{}", new_time)));
    }
}
