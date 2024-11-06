use gilrs::{GamepadId, Gilrs};
use tokio::{
    io::Result,
    net::UdpSocket,
    task::yield_now,
    time::{sleep, Duration},
};

mod controller;
mod controller_input;
mod logging;
mod networking;

use controller::{Button, ControllerState, StickState};

fn controller_test() {
    let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
        // println!("{:?}", id);
    }

    let mut active_id = gilrs.gamepads().next().unwrap().0;

    let mut controller_state = ControllerState::default();

    loop {
        let mut changed = false;

        let gamepad = gilrs.gamepad(active_id);
        // let new_controller_state = convert_gamepad(gamepad);
    }
}

async fn sender() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:55440").await?;

    let remote_addr = "10.137.4.10:55441";
    socket.connect(remote_addr).await?;

    let mut counter: u32 = 0;
    let delay = Duration::from_millis(1000);
    loop {
        sleep(delay).await;
        println!("Sending data: {counter}");
        let data = counter.to_le_bytes();
        let len = socket.send(&data).await?;
        println!("Sent {len} bytes");
        counter += 1;
    }
}

async fn listener() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:55441").await?;

    let remote_addr = "10.137.4.10:55440";
    socket.connect(remote_addr).await?;

    loop {
        let mut buf = [0u8; 32];
        let len = socket.recv(&mut buf).await?;
        println!("Received bytes: {:02X?}", &buf[..len]);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // logging::log_data().await
    // tokio::spawn(sender());
    // tokio::spawn(listener());

    // loop {
    //     yield_now().await;
    // }
    Ok(())
}
