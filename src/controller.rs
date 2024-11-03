#![allow(dead_code)]

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct StickState {
    x: i16,
    y: i16,
}

impl StickState {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: (x * 32768_f32) as i16,
            y: (y * 32768_f32) as i16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Cross,
    Circle,
    Square,
    Triangle,

    Up,
    Down,
    Left,
    Right,

    LeftBumper,
    RightBumper,

    L3,
    R3,

    Select,
    Start,

    Logo,
}

impl Button {
    fn mask(&self) -> u16 {
        1 << *self as usize
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct ControllerState {
    pub left_stick: StickState,
    pub right_stick: StickState,
    pub left_trigger: u8, // divided by 0x100 (256)
    pub right_trigger: u8,
    buttons: u16,
}

impl ControllerState {
    pub fn get(&self, button: Button) -> bool {
        self.buttons & button.mask() != 0
    }

    pub fn set(&mut self, button: Button) {
        self.buttons |= button.mask();
    }

    pub fn clear(&mut self, button: Button) {
        self.buttons &= !button.mask();
    }

    pub fn toggle(&mut self, button: Button) {
        self.buttons ^= button.mask();
    }

    pub fn set_left_trigger(&mut self, value: f32) {
        self.left_trigger = (value * 256_f32) as u8;
    }

    pub fn set_right_trigger(&mut self, value: f32) {
        self.right_trigger = (value * 256_f32) as u8;
    }
}