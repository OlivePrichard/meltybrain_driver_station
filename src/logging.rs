use chrono::Local;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, Result},
    sync::mpsc::Receiver,
};

pub async fn log_data(mut log_stream: Receiver<String>) -> Result<()> {
    let time = Local::now();
    let time_string = time.format("%Y-%m-%d_%H:%M:%S");
    let file_path = format!("logs/log_{time_string}.txt");
    let mut file = File::create_new(file_path).await?;

    while let Some(msg) = log_stream.recv().await {
        let result = file.write(msg.as_bytes()).await;
        if let Err(err) = result {
            file.write(err.to_string().as_bytes()).await?;
        }
    }

    Ok(())
}
