mod controller_input;
mod logging;
mod networking;
mod shared_code;

use shared_code::controller::ControllerState;

use gilrs::{GamepadId, Gilrs};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader, Result},
    sync::watch,
};

async fn get_gamepads(gilrs: &mut Gilrs) -> (Option<GamepadId>, Option<GamepadId>) {
    // let mut input = BufReader::new(stdin());

    println!("Press X on primary gamepad");

    let primary_gamepad_future = async {
        loop {
            if let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
                match event {
                    gilrs::EventType::ButtonPressed(gilrs::Button::South, _) => {
                        println!("Primary gamepad connected");
                        break id;
                    }
                    _ => (),
                }
            }
        }
    };
    let primary_gamepad = Some(primary_gamepad_future.await);

    println!("Press ○ on secondary gamepad");

    let secondary_gamepad_future = async {
        loop {
            if let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
                match event {
                    gilrs::EventType::ButtonPressed(gilrs::Button::East, _) => {
                        println!("Secondary gamepad connected");
                        break id;
                    }
                    _ => (),
                }
            }
        }
    };
    let secondary_gamepad = Some(secondary_gamepad_future.await);

    (primary_gamepad, secondary_gamepad)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (cancel_tx, cancel_rx) = watch::channel(false);
    let (controller_tx, controller_rx) =
        watch::channel((ControllerState::default(), ControllerState::default()));

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
    ));

    println!("Started");

    let mut input = BufReader::new(stdin());
    let mut buf = String::new();
    loop {
        input.read_line(&mut buf).await?;
        println!("{}", buf);
        if buf.trim() == "exit" {
            break;
        }
        buf.clear();
    }
    match cancel_tx.send(true) {
        Ok(_) => println!("Stopping!"),
        Err(_) => println!("Already stopped!"),
    }

    input_handle.await?;
    println!("Input stopped");
    networking_handle.await??;
    println!("Networking stopped");

    Ok(())
}
