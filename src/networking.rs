use crate::shared_code::{
    controller::ControllerState, log_messages::LogIterator, message_format::{Message, MessageIter},
};

use itertools::Itertools;
use std::{collections::VecDeque, io::Result, sync::Arc};
use tokio::{
    net::UdpSocket,
    sync::{mpsc::Sender, watch::Receiver, Mutex},
};

pub async fn handle_networking(
    cancel_signal: Receiver<bool>,
    inputs: Receiver<(ControllerState, ControllerState)>,
    logging: Sender<String>,
) -> Result<()> {
    // let mut socket = UdpSocket::bind("0.0.0.0").await?;
    let socket = UdpSocket::bind("0.0.0.0:55440").await?;

    // socket.connect("192.168.2.1:55440").await?;
    socket.connect("127.0.0.1:55440").await?;

    let socket = Arc::new(socket);

    let missing_logs = Arc::new(Mutex::new(VecDeque::new()));

    let receiver_handle = tokio::spawn(receiver(
        cancel_signal.clone(),
        socket.clone(),
        missing_logs.clone(),
        logging,
    ));
    let sender_handle = tokio::spawn(sender(
        cancel_signal.clone(),
        socket.clone(),
        missing_logs.clone(),
        inputs,
    ));

    receiver_handle.await??;
    sender_handle.await??;

    Ok(())
}

async fn sender(
    cancel_signal: Receiver<bool>,
    socket: Arc<UdpSocket>,
    missing_logs: Arc<Mutex<VecDeque<u32>>>,
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

        let logs = missing_logs.lock().await;
        let length = logs.len() * Message::MissedLogData(0).buffer_len()
            + Message::ControllerData(0, ControllerState::default(), ControllerState::default())
                .buffer_len();
        buffer.resize(length, 0u8);
        let buf = buffer.as_mut_slice();
        let mut len = 0;

        let (primary, secondary) = controllers;
        let message = Message::ControllerData(controller_message_counter, primary, secondary);
        controller_message_counter += 1;
        len += message.to_le_bytes(&mut buf[len..]) as usize;

        for id in logs.iter() {
            let message = Message::MissedLogData(*id);
            len += message.to_le_bytes(&mut buf[len..]) as usize;
        }
        drop(logs);

        socket.send(&buf[..len]).await?;
    }

    // let timeout = Instant::now() + Duration::from_secs(5);
    // while Instant::now() < timeout {
    //     tokio::time::sleep(Duration::from_millis(100)).await;

    //     let logs = missing_logs.lock().await;
    //     if logs.is_empty() {
    //         continue;
    //     }
    //     let length = logs.len() * Message::MissedLogData(0).buffer_len();
    //     buffer.resize(length, 0u8);
    //     let buf = buffer.as_mut_slice();
    //     let mut len = 0;

    //     for id in logs.iter() {
    //         let message = Message::MissedLogData(*id);
    //         len += message.to_le_bytes(&mut buf[len..]) as usize;
    //     }
    //     drop(logs);

    //     socket.send(&buf[..len]).await?;
    // }

    Ok(())
}

async fn receiver(
    cancel_signal: Receiver<bool>,
    socket: Arc<UdpSocket>,
    missing_logs: Arc<Mutex<VecDeque<u32>>>,
    logging: Sender<String>,
) -> Result<()> {
    let buffer = &mut [0u8; 0x1_00_00]; // Buffer is larger than the maximum sized UDP packet

    let mut next_packet = 0;

    let mut log_queue = VecDeque::new();

    while !*cancel_signal.borrow() {
        let len = socket.recv(buffer).await?;
        for message in MessageIter::new(&buffer[..len]) {
            match message {
                Message::ControllerData(..) => {
                    println!("How did this even happen?");
                }
                Message::LogData(id, buf) => {
                    if id > next_packet {
                        let mut logs = missing_logs.lock().await;
                        for i in next_packet..id {
                            logs.push_back(i);
                            log_queue.push_front(None);
                        }
                        next_packet = id + 1;
                        log_queue.push_front(Some(parse_log_data(id, buf)));
                    }
                    if id < next_packet {
                        let mut logs = missing_logs.lock().await;
                        if let Some(i) = logs.iter().position(|&x| x == id) {
                            logs.remove(i);
                            log_queue[next_packet as usize - id as usize - 1] =
                                Some(parse_log_data(id, buf));
                        }
                    } else {
                        log_queue.push_front(Some(parse_log_data(id, buf)));
                        next_packet += 1;
                    }
                }
                Message::MissedLogData(..) => {
                    println!("You're using messages wrong.");
                }
                Message::ForgotLogData(id) => {
                    let mut logs = missing_logs.lock().await;
                    if let Some(&i) = logs.iter().find(|&&x| x == id) {
                        logs.remove(i as usize);
                        log_queue[next_packet as usize - id as usize - 1] =
                            Some(format!("Lost log packet {}", id));
                    }
                }
            };
        }

        while let Some(Some(_)) = log_queue.back() {
            let log = log_queue.pop_back().unwrap().unwrap();
            if logging.capacity() == 0 {
                break;
            }
            _ = logging.send(log).await;
        }
    }

    for (log, id) in log_queue.into_iter().zip((0..next_packet).rev()) {
        _ = logging
            .send(match log {
                Some(data) => data,
                None => format!("Missed log {}", id),
            })
            .await;
    }

    Ok(())
}

fn parse_log_data(id: u32, data: &[u8]) -> String {
    let mut data_copy = data.to_vec();
    format!("Packet {}:\n{}", id, LogIterator::new(&mut data_copy)
        .map(|log| {
            format!(
                "[{}:{:02}.{:09}]: {}",
                log.time.as_secs() / 60,
                log.time.as_secs() % 60,
                log.time.subsec_nanos(),
                log.log.to_string()
            )
        })
        .join("\n"))
}
