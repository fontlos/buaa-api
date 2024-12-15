use std::time::{SystemTime, UNIX_EPOCH};
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

pub fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn get_primitive_time() -> PrimitiveDateTime {
    let now_utc = OffsetDateTime::now_utc();
    let local_offset = UtcOffset::from_hms(8, 0, 0).unwrap();
    let now_local = now_utc.to_offset(local_offset);
    PrimitiveDateTime::new(now_local.date(), now_local.time())
}
