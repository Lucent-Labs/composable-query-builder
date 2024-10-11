use std::fmt::{Display, Formatter};

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[repr(u8)]
pub enum OrderDir {
    Asc,
    Desc,
}

impl OrderDir {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderDir::Asc => "asc",
            OrderDir::Desc => "desc",
        }
    }
}

impl Display for OrderDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderDir::Asc => "asc",
                OrderDir::Desc => "desc",
            }
        )
    }
}
