mod controller_input;
mod logging;
mod networking;
mod shared_code;

use shared_code::controller::ControllerState;

use gilrs::{GamepadId, Gilrs};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader, Result}, sync::{mpsc, watch}
};

async fn read_line() -> Result<String> {
    let mut buf = String::new();
    let mut input = BufReader::new(stdin());
    input.read_line(&mut buf).await?;
    Ok(buf)
}

async fn get_gamepads(gilrs: &mut Gilrs) -> (Option<GamepadId>, Option<GamepadId>) {
    println!("Press X on primary gamepad");

    let mut possible_input = tokio::spawn(read_line());

    let primary_gamepad = loop {
        if let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(gilrs::Button::South, _) => {
                    println!("Primary gamepad connected");
                    break Some(id);
                }
                _ => (),
            }
        }

        if possible_input.is_finished() {
            if let Ok(Ok(input)) = possible_input.await {
                if input.trim() == "skip" {
                    break None;
                }
            }
            possible_input = tokio::spawn(read_line());
        }
    };

    println!("Press ○ on secondary gamepad");

    possible_input = tokio::spawn(read_line());

    let secondary_gamepad = loop {
        if let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(gilrs::Button::East, _) => {
                    println!("Secondary gamepad connected");
                    break Some(id);
                }
                _ => (),
            }
        }

        if possible_input.is_finished() {
            if let Ok(Ok(input)) = possible_input.await {
                if input.trim() == "skip" {
                    break None;
                }
            }
            possible_input = tokio::spawn(read_line());
        }
    };

    (primary_gamepad, secondary_gamepad)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (cancel_tx, cancel_rx) = watch::channel(false);
    let (controller_tx, controller_rx) =
        watch::channel((ControllerState::default(), ControllerState::default()));
    let (log_tx, log_rx) = mpsc::channel(1024 * 16);

    let mut gilrs = Gilrs::new().unwrap();
    let (primary_id, secondary_id) = get_gamepads(&mut gilrs).await;

    let input_handle = tokio::spawn(controller_input::read_controllers(
        cancel_rx.clone(),
        controller_tx,
        gilrs,
        primary_id,
        secondary_id,
    ));
    let networking_handle = tokio::spawn(networking::handle_networking(
        cancel_rx.clone(),
        controller_rx,
        log_tx,
    ));
    let logging_handle = tokio::spawn(logging::log_data(log_rx));

    loop {
        let input = read_line().await?;
        if input.trim() == "exit" {
            break;
        }
    }
    match cancel_tx.send(true) {
        Ok(_) => println!("Stopping!"),
        Err(_) => println!("Already stopped!"),
    }

    input_handle.await?;
    networking_handle.await??;
    logging_handle.await??;

    Ok(())
}
