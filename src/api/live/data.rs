use serde::Serialize;

/// Request Body Payload
pub enum Payload<'a, P: Serialize + ?Sized> {
    /// Query data
    Query(&'a P),
    /// JSON data
    Json(&'a P),
    /// No data
    Empty,
}
