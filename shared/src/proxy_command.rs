use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum PowerConfigSelection {
    Wired,
    Battery,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum ProxyCommand {
    ChangePowerConfig(PowerConfigSelection),
}
