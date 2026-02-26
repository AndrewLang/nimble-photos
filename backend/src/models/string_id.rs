use uuid::Uuid;

pub trait ToUuid {
    fn to_uuid(&self) -> Option<Uuid>;
}

impl ToUuid for String {
    fn to_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(self).ok()
    }
}
