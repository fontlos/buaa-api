use std::time::{SystemTime, UNIX_EPOCH};
use time::{Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

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

pub fn parse_date(time: &str) -> Date {
    let format_string = time::format_description::parse("[year]-[month]-[day]").unwrap();
    Date::parse(time, &format_string).unwrap()
}
