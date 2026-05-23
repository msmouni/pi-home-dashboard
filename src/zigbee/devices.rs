use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PlugDevice {
    pub id: String,
    pub name: String,

    pub state: Option<bool>,

    pub power: Option<f32>,
    pub voltage: Option<f32>,
    pub current: Option<f32>,
}
