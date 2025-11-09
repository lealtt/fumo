/// Formats an integer currency value using `.` as thousands separator
pub fn format_currency(value: i64) -> String {
    let mut number = value.abs().to_string();
    let mut formatted = String::new();

    while number.len() > 3 {
        let chunk = number.split_off(number.len() - 3);
        if formatted.is_empty() {
            formatted = chunk;
        } else {
            formatted = format!("{chunk}.{formatted}");
        }
    }

    if formatted.is_empty() {
        formatted = number;
    } else {
        formatted = format!("{number}.{formatted}");
    }

    if value < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}
