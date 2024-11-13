use rerun::external::ndarray::Order;
use tokio::task::Id;

use crate::controller::ControllerState;

pub enum Message<'a> {
    ControllerData(u32, ControllerState),
    LogData(u32, &'a [u8]),
    MissedLogData(u32),
}

impl<'a> Message<'a> {
    fn discriminant(&self) -> u32 {
        match self {
            Self::ControllerData(..) => 0,
            Self::LogData(..) => 1,
            Self::MissedLogData(..) => 2,
        }
    }

    fn to_le_bytes(&self, buffer: &mut [u8]) -> u32 {
        buffer[0..4].copy_from_slice(&self.discriminant().to_le_bytes());
        let len: u32 = match self {
            Self::ControllerData(id, state) => {
                buffer[8..12].copy_from_slice(&id.to_le_bytes());
                buffer[12..24].copy_from_slice(&state.to_le_bytes());
                24
            }
            Self::LogData(id, data) => {
                buffer[8..12].copy_from_slice(&id.to_le_bytes());
                buffer[12..(12 + data.len())].copy_from_slice(data);
                12 + data.len() as u32
            }
            Self::MissedLogData(id) => {
                buffer[8..12].copy_from_slice(&id.to_le_bytes());
                12
            }
        };
        buffer[4..8].copy_from_slice(&len.to_le_bytes());
        len
    }

    fn from_le_bytes(buffer: &'a [u8]) -> Option<Self> {
        let id = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
        match match u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) {
            0 => Self::ControllerData(0, ControllerState::default()),
            1 => Self::LogData(0, &[]),
            2 => Self::MissedLogData(0),
            _ => {
                return None;
            }
        } {
            Self::ControllerData(..) => Some(Self::ControllerData(
                id,
                ControllerState::from_le_bytes(buffer[12..24].try_into().unwrap()),
            )),
            Self::LogData(..) => Some(Self::LogData(
                id,
                &buffer
                    [12..u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize],
            )),
            Self::MissedLogData(..) => Some(Self::MissedLogData(id)),
        }
    }

    fn change_buffer<'b>(self, buffer: &'b mut [u8]) -> Message<'b> {
        if let Self::LogData(id, data) = self {
            buffer[..data.len()].copy_from_slice(data);
            return Message::LogData(id, &buffer[..data.len()]);
        }
        // these variants don't contain lifetime data
        unsafe { core::mem::transmute(self) }
    }
}
