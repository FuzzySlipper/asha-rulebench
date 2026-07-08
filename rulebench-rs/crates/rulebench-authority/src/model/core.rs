#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedValue {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedNumber {
    pub id: String,
    pub label: String,
    pub value: i32,
}
