use chrono::DateTime;

pub struct BaseEntity {
    updatedAt: DateTime,
    deletedAt: Option<DateTime>,
}
