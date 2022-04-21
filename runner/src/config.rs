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

impl Default for Speed {
    fn default() -> Self {
        Speed::Auto
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub interface: Interface,
    #[serde(default = "Default::default")]
    pub speed: Speed,
    pub target: String,
    pub target_description: Option<String>,
    #[serde(default = "Default::default")]
    pub vector_table_offset: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_config() {
        let json = r#"
        {
          "interface": "SWD",
          "target": "STM32F105"
        }
        "#;
        let config = serde_json::from_str::<Config>(json).expect("Parse minimal config");
        assert_eq!(config.vector_table_offset, 0);
        assert!(matches!(config.target_description, None));
        assert!(matches!(config.speed, Speed::Auto));
    }
}