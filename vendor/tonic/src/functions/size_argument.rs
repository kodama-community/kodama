use crate::ast::Measurement;
use crate::error::ParseError;

pub(crate) fn is_ascii_digit_code_unit(code: char) -> bool {
    code.is_ascii_digit()
}

pub(crate) fn size_scan_candidate(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut index = 0;
    if index < len && (chars[index] == '+' || chars[index] == '-') {
        index += 1;
    }
    index = skip_ascii_spaces(&chars, index);
    if index >= len {
        return true;
    }
    if is_ascii_digit_code_unit(chars[index]) {
        while index < len && is_ascii_digit_code_unit(chars[index]) {
            index += 1;
        }
        if index < len && chars[index] == '.' {
            index += 1;
            while index < len && is_ascii_digit_code_unit(chars[index]) {
                index += 1;
            }
        }
    } else if chars[index] == '.' {
        index += 1;
        while index < len && is_ascii_digit_code_unit(chars[index]) {
            index += 1;
        }
    } else {
        return false;
    }
    index = skip_ascii_spaces(&chars, index);
    let unit_start = index;
    while index < len && chars[index].is_ascii_lowercase() {
        index += 1;
    }
    if index - unit_start > 2 {
        return false;
    }
    index = skip_ascii_spaces(&chars, index);
    index == len
}

pub(crate) fn skip_ascii_spaces(chars: &[char], start: usize) -> usize {
    let mut index = start;
    while index < chars.len() && chars[index] == ' ' {
        index += 1;
    }
    index
}

pub(crate) fn parse_decimal(text: &str) -> Result<f64, ParseError> {
    text.parse::<f64>()
        .map_err(|_| ParseError::InternalInvariant {
            message: format!("Validated size number failed Double conversion: {text}"),
        })
}

pub(crate) fn parse_size_measurement(text: &str) -> Result<Option<Measurement>, ParseError> {
    // KaTeX's final measurement regex is intentionally unanchored.
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut start = 0;
    while start < len {
        let mut index = start;
        let mut number_builder = String::new();
        if chars[index] == '+' || chars[index] == '-' {
            number_builder.push(chars[index]);
            index += 1;
            index = skip_ascii_spaces(&chars, index);
        }
        let number_start = index;
        while index < len && is_ascii_digit_code_unit(chars[index]) {
            number_builder.push(chars[index]);
            index += 1;
        }
        let digits_before_dot = index - number_start;
        let mut digits_after_dot = 0;
        if index < len && chars[index] == '.' {
            number_builder.push('.');
            index += 1;
            let fraction_start = index;
            while index < len && is_ascii_digit_code_unit(chars[index]) {
                number_builder.push(chars[index]);
                index += 1;
            }
            digits_after_dot = index - fraction_start;
        }
        if digits_before_dot > 0 || digits_after_dot > 0 {
            index = skip_ascii_spaces(&chars, index);
            if index + 2 <= len {
                let first = chars[index];
                let second = chars[index + 1];
                if first.is_ascii_lowercase() && second.is_ascii_lowercase() {
                    let unit: String = chars[index..index + 2].iter().collect();
                    let number = parse_decimal(&number_builder)?;
                    return Ok(Some(Measurement { number, unit }));
                }
            }
        }
        start += 1;
    }
    Ok(None)
}

pub(crate) fn valid_size_unit(unit: &str) -> bool {
    matches!(
        unit,
        "pt" | "mm"
            | "cm"
            | "in"
            | "bp"
            | "pc"
            | "dd"
            | "cc"
            | "nd"
            | "nc"
            | "sp"
            | "px"
            | "ex"
            | "em"
            | "mu"
    )
}
