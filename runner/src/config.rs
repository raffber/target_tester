use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Interface {
    JTAG,
    SWD,
}

#[derive(Serialize, Deserialize)]
pub enum Speed {
    Auto,
    Adaptive,
    KHz(u16),
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    interface: Interface,
    speed_khz: u16,
}
