use serde_derive::Serialize;

pub mod port;
pub mod storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    File,
    Api,
}
