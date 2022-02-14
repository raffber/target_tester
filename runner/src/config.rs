use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Interface {
    JTAG,
    SWD,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    interface: Interface,
    speed_khz: u16,

}
