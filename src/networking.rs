use crate::{
    logging::log_data,
    shared_code::{controller::ControllerState, log_messages::Log, message_format::Message},
};

use itertools::Itertools;
use std::{io::Result, sync::Arc};
use tokio::{net::UdpSocket, sync::watch::Receiver};

pub async fn handle_networking(
    cancel_signal: Receiver<bool>,
    inputs: Receiver<(ControllerState, ControllerState)>,
) -> Result<()> {
    // let mut socket = UdpSocket::bind("0.0.0.0").await?;
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    socket.connect("192.168.2.1:55440").await?;
    // socket.connect("127.0.0.1:55440").await?;

    let socket = Arc::new(socket);

    let logs = Vec::new();

    let receiver_handle = tokio::spawn(receiver(cancel_signal.clone(), socket.clone(), logs));
    let sender_handle = tokio::spawn(sender(cancel_signal.clone(), socket.clone(), inputs));

    receiver_handle.await??;
    sender_handle.await??;

    Ok(())
}

async fn sender(
    cancel_signal: Receiver<bool>,
    socket: Arc<UdpSocket>,
    mut inputs: Receiver<(ControllerState, ControllerState)>,
) -> Result<()> {
    let mut controller_message_counter = 0;
    let mut buffer = Vec::new();

    while !*cancel_signal.borrow() {
        // we receive controller inputs at 50Hz
        if inputs.changed().await.is_err() {
            break;
        }
        let controllers = *inputs.borrow_and_update();

        let length =
            Message::ControllerData(0, ControllerState::default(), ControllerState::default())
                .buffer_len();
        buffer.resize(length, 0u8);
        let buf = buffer.as_mut_slice();
        let mut len = 0;

        let (primary, secondary) = controllers;
        let message = Message::ControllerData(controller_message_counter, primary, secondary);
        controller_message_counter += 1;
        len += message.to_le_bytes(&mut buf[len..]) as usize;

        socket.send(&buf[..len]).await?;
    }

    Ok(())
}

async fn receiver(
    cancel_signal: Receiver<bool>,
    socket: Arc<UdpSocket>,
    mut logs: Vec<Option<String>>,
) -> Result<()> {
    let buffer = &mut [0u8; 0x1_00_00]; // Buffer is larger than the maximum sized UDP packet

    while !*cancel_signal.borrow() {
        let len = socket.recv(buffer).await?;
        let id = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
        let mut data = &buffer[4..len];
        let mut message_vec = Vec::new();
        while !data.is_empty() {
            if data.len() < 6 {
                break;
            }
            let length = data[0] as usize;
            let log = Log::from_bytes(&data);
            match log {
                Some(log) => {
                    message_vec.push(log);
                    data = &data[length..];
                }
                None => {
                    if data.len() <= length {
                        break;
                    } else {
                        data = &data[length..];
                    }
                }
            }
        }

        if logs.len() <= id as usize {
            logs.resize(id as usize + 1, None);
        }
        logs[id as usize] = Some(format!(
            "Packet {}:\n{}\n",
            id,
            message_vec
                .into_iter()
                .map(|log| {
                    format!(
                        "[{}:{:02}.{:03}_{:03}]: {}",
                        log.time.as_secs() / 60,
                        log.time.as_secs() % 60,
                        log.time.subsec_millis(),
                        log.time.subsec_micros() % 100,
                        log.log.to_string()
                    )
                })
                .join("\n")
        ));
    }

    log_data(logs).await?;

    Ok(())
}
