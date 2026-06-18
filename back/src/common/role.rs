#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    CoOwner,
    CoOwnershipBoard,
    CoOwnershipBoardOps,
}

impl Role {
    pub const ALL: [Self; 4] = [Self::Admin, Self::CoOwner, Self::CoOwnershipBoard, Self::CoOwnershipBoardOps];
    pub const ASSIGNABLE: [Self; 3] = [Self::CoOwner, Self::CoOwnershipBoard, Self::CoOwnershipBoardOps];

    pub fn code(self) -> &'static str {
        match self {
            Self::Admin => "ADMIN",
            Self::CoOwner => "CO_OWNER",
            Self::CoOwnershipBoard => "CO_OWNERSHIP_BOARD",
            Self::CoOwnershipBoardOps => "CO_OWNERSHIP_BOARD_OPS",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Admin => "roles.admin",
            Self::CoOwner => "roles.coOwner",
            Self::CoOwnershipBoard => "roles.coOwnershipBoard",
            Self::CoOwnershipBoardOps => "roles.coOwnershipBoardOps",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ADMIN" => Some(Self::Admin),
            "CO_OWNER" => Some(Self::CoOwner),
            "CO_OWNERSHIP_BOARD" => Some(Self::CoOwnershipBoard),
            "CO_OWNERSHIP_BOARD_OPS" => Some(Self::CoOwnershipBoardOps),
            _ => None,
        }
    }

    pub fn is_assignable(self) -> bool {
        Self::ASSIGNABLE.contains(&self)
    }
}
