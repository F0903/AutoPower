use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum RefreshRateMode {
    Max,
    Value(u32),
    Min,
}
