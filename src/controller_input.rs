use crate::controller::{Button, ControllerState, StickState};
use gilrs::{Gamepad, GamepadId, Gilrs};

fn convert_gamepad(gamepad: Gamepad<'_>) -> ControllerState {
    let mut controller_state = ControllerState::default();

    if gamepad.is_pressed(gilrs::Button::South) {
        controller_state.set(Button::Cross);
    }
    if gamepad.is_pressed(gilrs::Button::East) {
        controller_state.set(Button::Circle);
    }
    if gamepad.is_pressed(gilrs::Button::West) {
        controller_state.set(Button::Square);
    }
    if gamepad.is_pressed(gilrs::Button::North) {
        controller_state.set(Button::Triangle);
    }

    if gamepad.is_pressed(gilrs::Button::DPadUp) {
        controller_state.set(Button::Up);
    }
    if gamepad.is_pressed(gilrs::Button::DPadDown) {
        controller_state.set(Button::Down);
    }
    if gamepad.is_pressed(gilrs::Button::DPadLeft) {
        controller_state.set(Button::Left);
    }
    if gamepad.is_pressed(gilrs::Button::DPadRight) {
        controller_state.set(Button::Right);
    }

    if gamepad.is_pressed(gilrs::Button::LeftTrigger) {
        controller_state.set(Button::LeftBumper);
    }
    if gamepad.is_pressed(gilrs::Button::RightTrigger) {
        controller_state.set(Button::RightBumper);
    }

    if gamepad.is_pressed(gilrs::Button::LeftThumb) {
        controller_state.set(Button::L3);
    }
    if gamepad.is_pressed(gilrs::Button::RightThumb) {
        controller_state.set(Button::R3);
    }

    if gamepad.is_pressed(gilrs::Button::Select) {
        controller_state.set(Button::Select);
    }
    if gamepad.is_pressed(gilrs::Button::Start) {
        controller_state.set(Button::Start);
    }
    if gamepad.is_pressed(gilrs::Button::Mode) {
        controller_state.set(Button::Logo);
    }

    controller_state.set_left_trigger(
        gamepad
            .axis_data(gilrs::Axis::LeftZ)
            .map_or(0f32, |axis| axis.value()),
    );
    controller_state.set_right_trigger(
        gamepad
            .axis_data(gilrs::Axis::RightZ)
            .map_or(0f32, |axis| axis.value()),
    );

    controller_state.left_stick = StickState::new(
        gamepad
            .axis_data(gilrs::Axis::LeftStickX)
            .map_or(0f32, |axis| axis.value()),
        gamepad
            .axis_data(gilrs::Axis::LeftStickY)
            .map_or(0f32, |axis| axis.value()),
    );
    controller_state.right_stick = StickState::new(
        gamepad
            .axis_data(gilrs::Axis::RightStickX)
            .map_or(0f32, |axis| axis.value()),
        gamepad
            .axis_data(gilrs::Axis::RightStickY)
            .map_or(0f32, |axis| axis.value()),
    );

    controller_state
}

pub fn get_controller_state(gilrs: Gilrs, id: GamepadId) -> ControllerState {
    convert_gamepad(gilrs.gamepad(id))
}
