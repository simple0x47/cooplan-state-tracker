use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum State {
    Idle,
    Valid,
    Error(String),
}
