use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefreshRateMode {
    Max,
    Value(u32),
    Min,
}
