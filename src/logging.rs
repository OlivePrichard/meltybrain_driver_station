use chrono::Local;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, Result},
};

pub async fn log_data(log_vec: Vec<Option<String>>) -> Result<()> {
    let time = Local::now();
    let time_string = time.format("%Y-%m-%d_%H:%M:%S");
    let file_path = format!("logs/log_{time_string}.txt");
    let mut file = File::create_new(file_path).await?;

    for (i, log) in log_vec.into_iter().enumerate() {
        let msg = match log {
            Some(msg) => msg,
            None => format!("Packet {} not received!\n", i),
        };
        let result = file.write(msg.as_bytes()).await;
        if let Err(err) = result {
            file.write(err.to_string().as_bytes()).await?;
        }
    }

    Ok(())
}
