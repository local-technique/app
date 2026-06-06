#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    CoOwner,
}

impl Role {
    pub const ALL: [Self; 2] = [Self::Admin, Self::CoOwner];
    pub const ASSIGNABLE: [Self; 1] = [Self::CoOwner];

    pub fn code(self) -> &'static str {
        match self {
            Self::Admin => "ADMIN",
            Self::CoOwner => "CO_OWNER",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Admin => "roles.admin",
            Self::CoOwner => "roles.coOwner",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ADMIN" => Some(Self::Admin),
            "CO_OWNER" => Some(Self::CoOwner),
            _ => None,
        }
    }

    pub fn is_assignable(self) -> bool {
        Self::ASSIGNABLE.contains(&self)
    }
}
