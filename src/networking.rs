use crate::shared_code::{controller::ControllerState, message_format::Message};

use std::{io::Result, sync::Arc};
use tokio::{net::UdpSocket, sync::watch::Receiver};

pub async fn handle_networking(
    cancel_signal: Receiver<bool>,
    inputs: Receiver<(ControllerState, ControllerState)>,
) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    socket.connect("192.168.2.1:55440").await?;

    let socket = Arc::new(socket);

    sender(cancel_signal.clone(), socket.clone(), inputs).await?;

    Ok(())
}

async fn sender(
    cancel_signal: Receiver<bool>,
    socket: Arc<UdpSocket>,
    mut inputs: Receiver<(ControllerState, ControllerState)>,
) -> Result<()> {
    let mut controller_message_counter = 0;
    let buffer = &mut [0u8; 36][..];

    while !*cancel_signal.borrow() {
        // we receive controller inputs at 50Hz
        if inputs.changed().await.is_err() {
            break;
        }
        let controllers = *inputs.borrow_and_update();

        let (primary, secondary) = controllers;
        let message = Message::ControllerData(controller_message_counter, primary, secondary);
        controller_message_counter += 1;
        let len = message.to_le_bytes(buffer) as usize;

        socket.send(&buffer[..len]).await?;
    }

    Ok(())
}
