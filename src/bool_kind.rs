#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum BoolKind {
    #[default]
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
