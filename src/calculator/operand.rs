use rust_decimal::{Decimal, prelude::FromPrimitive};

use super::{is_number, BUFFER_SIZE, DOT_SYMBOL, CommandError, BUFFER_MAX_NUMBER_DEC, BUFFER_MAX_NUMBER};
use std::str::FromStr;

const BUFFER_SIZE_INTERN: usize = 28 + 1;

/// Implements operand representation and conversion to current view for calculator
#[derive(Debug, Clone)]
pub struct Operand {
    buffer: String,
    dot_is_after: Option<usize>,
    is_negative: bool,
    reset_on_clear: bool,
}

impl Operand {
    pub(super) fn new() -> Self {
        Default::default()
    }

    pub(super) fn send_symbol(&mut self, symbol: char) -> bool {
        if !self.has_free_space() {
            return false;
        }
        if self.dot_is_after.is_none() && Self::is_dot(symbol) {
            self.dot_is_after = Some(self.buffer.chars().count() - 1);
            return true;
        }
        if is_number(symbol) {
            if self.is_internally_empty() && !self.dot_is_here() {
                self.buffer.clear();
            }
            self.buffer.push(symbol);
            return true;
        }
        return false;
    }

    pub(super) fn _set_negative(&mut self) -> bool {
        if self.is_negative {
            return false
        }
        self.is_negative = true;
        return true
    }

    pub(super) fn set_should_reset_on_clear(&mut self, value: bool) {
        self.reset_on_clear = value;
    }

    pub(super) fn should_reset_on_clear(&self) -> bool {
        self.reset_on_clear
    }

    pub fn is_positive(&self) -> bool {
        !self.is_negative
    }

    pub(super) fn send_erase(&mut self) -> bool {
        if self.reset_on_clear {
            self.clean_entirely();
            return true;
        }
        if self.dot_is_here() {
            self.dot_is_after = None;
            return true;
        }
        if self.is_internally_empty() {
            return false;
        }
        self.buffer.pop();
        if self.buffer.is_empty() {
            self.buffer.push('0');
            self.is_negative = false;
        }
        return true;
    }

    fn clean_entirely(&mut self) {
        self.dot_is_after = None;
        self.is_negative = false;
        self.buffer.clear();
        self.buffer.push('0');
    }

    pub fn numbers_count(&self) -> usize {
        self.buffer.len().min(BUFFER_SIZE)
    }

    pub fn has_dot(&self) -> bool {
        self.dot_is_after.is_some()
    }

    fn has_dot_after(&self, index: usize) -> bool {
        matches!(self.dot_is_after, Some(dot_is_after_pos)
                                    if dot_is_after_pos == index)
    }

    fn dot_is_here(&self) -> bool {
        let numbers_count: usize = self.buffer.chars().count();
        let last_index: usize = if numbers_count >= 1 {
            numbers_count - 1
        } else {
            0
        };
        self.has_dot_after(last_index)
    }

    fn has_free_space(&self) -> bool {
        self.buffer.chars().count() < BUFFER_SIZE
    }

    fn buffer_is_full(&self) -> bool {
        !self.has_free_space()
    }

    pub fn is_internally_empty(&self) -> bool {
        self.buffer == "0"
    }

    pub fn is_externally_empty(&self) -> bool {
        self.is_internally_empty() && self.dot_is_after.is_none()
    }

    pub fn current_state_string(&self) -> String {
        let is_dot_after_last_number = |dot_is_after_pos: usize| 
            !self.buffer_is_full() && dot_is_after_pos == self.buffer.chars().count() - 1;
        let mut basic_str: String = self.to_string();
        if matches!(self.dot_is_after, Some(dot_is_after_pos)
                                       if is_dot_after_last_number(dot_is_after_pos)) {
            basic_str.push(DOT_SYMBOL);
        }
        let basic_str_len: usize = basic_str.chars().count();
        let digits_count: usize = basic_str_len - {
            let mut digits_not_to_count: usize = 0;
            if basic_str.starts_with('0') {
                digits_not_to_count += 1;
            }
            if basic_str.contains('-') {
                digits_not_to_count += 1;
            }
            if basic_str.contains('.') {
                digits_not_to_count += 1;
            }
            digits_not_to_count
        };
        let extra_digits_count: usize = digits_count - BUFFER_SIZE.min(digits_count);
        for _ in 0..extra_digits_count {
            basic_str.pop();
        }
        return basic_str;
    }

    fn is_dot(symbol: char) -> bool {
        symbol == '.'|| symbol == ','
    }
}

impl Default for Operand {
    fn default() -> Self {
        let mut buffer = String::with_capacity(29);
        buffer.push('0');
        Self {
            buffer,
            dot_is_after: None,
            is_negative: false,
            reset_on_clear: false,
        }
    }
}

#[test]
fn test_powers() {
    let a: f64 = 128.;
    let b: f64 = 2.;
    let result: f64 = a.ln() / b.ln();
    dbg!(result);
}

impl ToString for Operand {
    fn to_string(&self) -> String {
        const DOT_SIZE: usize = 1;
        const MINUS_SIGN_SIZE: usize = 1;
        let mut s = String::with_capacity(BUFFER_SIZE + DOT_SIZE + MINUS_SIGN_SIZE);
        if self.is_negative {
            s.push('-');
        }
        if self.dot_is_after == None || self.dot_is_after.unwrap() == self.buffer.chars().count() - 1 {
            s.push_str(&self.buffer);
            return s;
        }
        for (current_pos, ch) in self.buffer.chars().enumerate() {
            s.push(ch);
            if matches!(self.dot_is_after, Some(dot_is_after_pos) 
                                           if dot_is_after_pos == current_pos) {
                s.push(DOT_SYMBOL);
            }
        }
        return s;
    }
}

#[test]
fn t_dec_max() {
    let mut max_dec = Decimal::MAX;
    max_dec.rescale(27);
    let dec_max_str: String = max_dec.to_string();
    dbg!(&dec_max_str);
    dbg!(dec_max_str.chars().count());
}

#[test]
fn t_general() {
    // let number = dec!();
}

impl TryFrom<Decimal> for Operand {
    type Error = CommandError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        if value.trunc().abs() > BUFFER_MAX_NUMBER_DEC {
            Err(CommandError::IncorrectOperand(value))?
        }
        let raw_str: String = value.to_string().replace("-", "");
        Ok(raw_str_to_buffer(raw_str, value.is_sign_negative()))
    }
}

impl TryFrom<f64> for Operand {
    type Error = CommandError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.trunc().abs() > BUFFER_MAX_NUMBER {
            Err(CommandError::IncorrectOperand(Decimal::from_f64(value).unwrap()))?
        }
        let raw_str: String = value.to_string().replace("-", ""); // allocation
        Ok(raw_str_to_buffer(raw_str, value < 0.0))
    }
}

fn raw_str_to_buffer(raw_str: String, is_negative: bool) -> Operand {
    let total_chars_count: usize = raw_str.chars().count();
    let last_index: usize = total_chars_count.min(BUFFER_SIZE_INTERN);
    let dot_pos: Option<usize> = raw_str[0..last_index].find('.');
    let mut buffer = String::with_capacity(BUFFER_SIZE_INTERN); // allocation
    if let Some(dot_pos) = dot_pos {
        let basic_str: &str = &raw_str[0..last_index]; ////
        buffer.extend(basic_str[0..dot_pos].chars());
        buffer.extend(basic_str[dot_pos + 1..].chars());
    } else {
        let basic_str: &str = &raw_str[0..last_index];
        buffer.extend(basic_str.chars());
    }
    Operand {
        buffer,
        dot_is_after: dot_pos.map(|dot_pos| dot_pos - 1),
        is_negative,
        ..Default::default()
    }
}

impl TryInto<Decimal> for Operand {
    type Error = CommandError;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        let s: String = self.to_string();
        Ok(Decimal::from_str(&s)
            .map_err(|_| CommandError::ParsingFailure { err: None })?)
    }
}

impl TryInto<Decimal> for &Operand {
    type Error = CommandError;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        let s: String = self.to_string();
        Ok(Decimal::from_str(&s)
            .map_err(|_| CommandError::ParsingFailure { err: None })?)
    }
}

impl TryInto<f64> for Operand {
    type Error = CommandError;

    fn try_into(self) -> Result<f64, Self::Error> {
        let s: String = self.to_string();
        Ok(f64::from_str(&s)
            .map_err(|err| CommandError::ParsingFailure { err: Some(err) })?)
    }
}