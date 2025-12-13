use ai_core::finance::{Money, Percentage};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub struct FormattedMoney(pub Money);

#[derive(Clone, PartialEq, Debug)]
pub struct FormattedPercentage(pub Percentage);

impl FormattedMoney {
    pub fn from_money(money: Money) -> Self {
        Self(money)
    }
}

impl fmt::Display for FormattedMoney {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = format!("{:.2}", self.0.value);
        let parts: Vec<&str> = formatted.split('.').collect();

        let integer_part = parts[0];
        let decimal_part = if parts.len() > 1 { parts[1] } else { "00" };

        // Добавляем пробелы как разделители тысяч
        let integer_with_spaces = add_thousand_separators(integer_part, '\'');

        // Если дробная часть "00", не показываем
        let value_str = if decimal_part == "00" {
            integer_with_spaces
        } else {
            format!("{}.{}", integer_with_spaces, decimal_part)
        };

        write!(f, "{}{}", value_str, self.0.currency)
    }
}

impl From<Money> for FormattedMoney {
    fn from(money: Money) -> Self {
        Self(money)
    }
}

impl FormattedPercentage {
    pub fn from_percentage(percentage: Percentage) -> Self {
        Self(percentage)
    }
}

impl fmt::Display for FormattedPercentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Percentage> for FormattedPercentage {
    fn from(percentage: Percentage) -> Self {
        Self(percentage)
    }
}

/// Добавляет пробелы как разделители тысяч
fn add_thousand_separators(s: &str, sep: char) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(sep);
        }
        result.push(*ch);
    }

    result
}
