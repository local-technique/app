#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    CoOwner,
    CoOwnershipBoard,
}

impl Role {
    pub const ALL: [Self; 3] = [Self::Admin, Self::CoOwner, Self::CoOwnershipBoard];
    pub const ASSIGNABLE: [Self; 2] = [Self::CoOwner, Self::CoOwnershipBoard];

    pub fn code(self) -> &'static str {
        match self {
            Self::Admin => "ADMIN",
            Self::CoOwner => "CO_OWNER",
            Self::CoOwnershipBoard => "CO_OWNERSHIP_BOARD",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Admin => "roles.admin",
            Self::CoOwner => "roles.coOwner",
            Self::CoOwnershipBoard => "roles.coOwnershipBoard",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ADMIN" => Some(Self::Admin),
            "CO_OWNER" => Some(Self::CoOwner),
            "CO_OWNERSHIP_BOARD" => Some(Self::CoOwnershipBoard),
            _ => None,
        }
    }

    pub fn is_assignable(self) -> bool {
        Self::ASSIGNABLE.contains(&self)
    }
}
