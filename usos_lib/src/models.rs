use serde::{Deserialize, Serialize};

pub struct TimetableUserStart {}

impl TimetableUserStart {
    pub fn to_string(&self) -> String {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsosTimetableUser {}