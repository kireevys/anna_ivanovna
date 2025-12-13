use ai_core::finance::Money;
use rust_decimal::Decimal;

pub fn format_money(value: &Money) -> String {
    format!("{}{}", value.value, value.currency)
}
/// Форматирует Decimal с разделителями тысяч (пробел) и 2 знаками после запятой
/// Пример: 375000.0 -> "375 000"
pub fn format_decimal(value: &Decimal) -> String {
    let formatted = format!("{:.2}", value);
    let parts: Vec<&str> = formatted.split('.').collect();

    let integer_part = parts[0];
    let decimal_part = if parts.len() > 1 { parts[1] } else { "00" };

    // Добавляем пробелы как разделители тысяч
    let integer_with_spaces = add_thousand_separators(integer_part);

    // Если дробная часть "00", не показываем
    if decimal_part == "00" {
        integer_with_spaces
    } else {
        format!("{}.{}", integer_with_spaces, decimal_part)
    }
}

/// Добавляет пробелы как разделители тысяч
fn add_thousand_separators(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(' ');
        }
        result.push(*ch);
    }

    result
}
