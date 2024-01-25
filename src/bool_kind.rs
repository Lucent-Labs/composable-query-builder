#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BoolKind {
    And,
    Or,
}

impl BoolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BoolKind::And => "and",
            BoolKind::Or => "or",
        }
    }
}
