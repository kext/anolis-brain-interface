#![no_std]

extern crate alloc;

mod packet;
pub use packet::*;
mod l2cap_error;
pub use l2cap_error::*;

pub const PSM: u16 = 0x2349;
pub const QUEUE_SIZE: u8 = 200;
