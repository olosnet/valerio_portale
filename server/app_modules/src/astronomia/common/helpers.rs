pub fn secs_to_dms(secs: i32) -> (i32, i32, i32) {
    let degrees = secs / 3600;
    let remainder = secs % 3600;
    let minutes = remainder / 60;
    let seconds = remainder % 60;

    (degrees, minutes, seconds)
}

pub fn secs_to_dms_string(value: i32) -> String {
    let (degrees, minutes, seconds) = secs_to_dms(value);
    format!("{:02}{:02}{:02}", degrees, minutes, seconds)
}
