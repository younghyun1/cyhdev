use uuid::Uuid;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoleType {
    Younghyun = 0,
    Moderator = 1,
    User = 2,
    Guest = 3,
}

impl RoleType {
    pub fn from_uuid(role_id: Uuid) -> Option<RoleType> {
        match role_id.as_u128() {
            ROLE_YOUNGHYUN => Some(RoleType::Younghyun),
            ROLE_MODERATOR => Some(RoleType::Moderator),
            ROLE_USER => Some(RoleType::User),
            ROLE_GUEST => Some(RoleType::Guest),
            _ => None,
        }
    }

    pub fn id(self) -> Uuid {
        match self {
            RoleType::Younghyun => Uuid::from_u128(ROLE_YOUNGHYUN),
            RoleType::Moderator => Uuid::from_u128(ROLE_MODERATOR),
            RoleType::User => Uuid::from_u128(ROLE_USER),
            RoleType::Guest => Uuid::from_u128(ROLE_GUEST),
        }
    }

    pub fn is_superuser(self) -> bool {
        self == Self::Younghyun
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Younghyun => "younghyun",
            Self::Moderator => "moderator",
            Self::User => "user",
            Self::Guest => "guest",
        }
    }

    pub fn permits(self, required_role_type: RoleType) -> bool {
        self.access_level() >= required_role_type.access_level()
    }

    fn access_level(self) -> u8 {
        match self {
            RoleType::Younghyun => 3,
            RoleType::Moderator => 2,
            RoleType::User => 1,
            RoleType::Guest => 0,
        }
    }
}

// 019a6c86-8bca-7b91-b9c0-1d4cc96b3263
const ROLE_YOUNGHYUN: u128 = 2131042872073453539493660941469037155;
// 019a6c86-b163-7452-aa70-5997736b0434
const ROLE_MODERATOR: u128 = 2131042883709330333470894399469323316;
// 019a6c86-bfa6-7903-9176-dc5f66f729fe
const ROLE_USER: u128 = 2131042888123140653623930835701279230;
// 019a6c86-d66b-7223-97ef-a8a26551a080
const ROLE_GUEST: u128 = 2131042895169936790354381715792830592;

#[cfg(test)]
mod tests {
    use super::RoleType;
    use uuid::Uuid;

    #[test]
    fn role_ids_round_trip() {
        for role in [
            RoleType::Younghyun,
            RoleType::Moderator,
            RoleType::User,
            RoleType::Guest,
        ] {
            assert_eq!(RoleType::from_uuid(role.id()), Some(role));
        }
    }

    #[test]
    fn unknown_role_id_is_rejected() {
        assert_eq!(RoleType::from_uuid(Uuid::nil()), None);
    }

    #[test]
    fn permissions_follow_role_hierarchy() {
        assert!(RoleType::Younghyun.permits(RoleType::Moderator));
        assert!(RoleType::Moderator.permits(RoleType::User));
        assert!(!RoleType::User.permits(RoleType::Moderator));
        assert!(!RoleType::Guest.is_superuser());
    }
}
