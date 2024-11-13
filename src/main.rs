use gilrs::{GamepadId, Gilrs};
use tokio::{
    io::Result,
    net::UdpSocket,
    task::yield_now,
    time::{sleep, Duration, Instant},
};

mod controller;
mod controller_input;
mod logging;
mod packet_formatting;

use controller::{Button, ControllerState, StickState};

async fn sender() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:55440").await?;

    let remote_addr = "192.168.169.113:55440";
    socket.connect(remote_addr).await?;

    let mut counter: u32 = 0;
    let delay = Duration::from_millis(1000);
    loop {
        sleep(delay).await;
        println!("Sending data: {counter}");
        let data = counter.to_le_bytes();
        let start = Instant::now();
        let len = socket.send(&data).await?;
        println!("Sent {len} bytes");
        counter += 1;

        let mut buf = [0u8; 32];
        let len = socket.recv(&mut buf).await?;
        let end = Instant::now();
        println!("Received bytes: {:02X?}", &buf[..len]);
        let round_trip = end - start;
        println!("Round trip took {} ms", round_trip.as_millis());
    }
}

async fn listener() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:55441").await?;

    let remote_addr = "192.168.2.1:55440";
    socket.connect(remote_addr).await?;

    loop {
        let mut buf = [0u8; 32];
        let len = socket.recv(&mut buf).await?;
        println!("Received bytes: {:02X?}", &buf[..len]);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut gilrs = Gilrs::new().unwrap();

    println!("Press X on primary gamepad");

    let primary_gamepad = loop {
        if let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(gilrs::Button::South, _) => {
                    println!("Primary gamepad connected");
                    break id;
                }
                _ => (),
            }
        }
    };

    println!("Press ○ on secondary gamepad");

    let secondary_gamepad = loop {
        if let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(gilrs::Button::East, _) => {
                    println!("Secondary gamepad connected");
                    break id;
                }
                _ => (),
            }
        }
    };

    // logging::log_data().await
    tokio::spawn(sender());
    tokio::spawn(listener());
    
    loop {
        yield_now().await;
    }
}
