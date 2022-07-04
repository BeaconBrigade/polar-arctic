use super::setting::Setting;
use crate::menu::Meta;
use tokio::{
    fs::OpenOptions,
    io::{AsyncWriteExt, BufWriter, Error},
};

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
pub async fn init(Setting { hr, ecg, acc }: Setting, metadata: Meta) -> Result<(), Error> {
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
async fn add_headers(ty: MeasureType, path: &str, msg: String) -> Result<(), Error> {
    let output = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .await?;
    let mut writer = BufWriter::with_capacity(200, output);
    let mut message = msg;
    message.push_str(&ty.to_string());

    writer.write_all(message.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}
