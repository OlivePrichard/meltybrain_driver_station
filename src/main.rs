use gilrs::{Event, Gamepad, Gilrs};

mod controller;

use controller::{ControllerState, Button};

fn convert_gamepad(gamepad: Gamepad<'_>) -> ControllerState {
    let mut controller_state = ControllerState::default();

    // todo add the rest of this code
    if gamepad.is_pressed(gilrs::Button::South) {
        controller_state.set(Button::Cross);
    }

    controller_state
}

fn main() {
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
        let new_controller_state = convert_gamepad(gamepad);
    }
}
