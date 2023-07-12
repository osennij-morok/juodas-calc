use std::fmt::Debug;

use rust_decimal::{Decimal, MathematicalOps};

use crate::calculator::BUFFER_SIZE;

use super::{CommandError, BUFFER_MAX_NUMBER_DEC};


#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Division,
    Multiplication,
    Addition,
    Subtraction,
    Power,
    NaturalLogarithm,
    Sine,
    Cosine,
}

impl Operator {
    pub fn apply(&self, left: Decimal, right: Option<Decimal>) -> Result<Decimal, CommandError> {
        println!(); ////
        println!("OPERATOR:"); ////
        dbg!(self); ////
        println!("CALCULATION:\n{} and {} =", left, right.unwrap_or(Decimal::ZERO)); ////
        let result: Result<Decimal, CommandError> = match self {
            Operator::Division => self.valid_result(
                left.checked_div(require_operand(right)?)
                    .ok_or(CommandError::Overflow)?),
            Operator::Multiplication => self.valid_result(
                left.checked_mul(require_operand(right)?)
                    .ok_or(CommandError::Overflow)?),
            Operator::Addition => self.valid_result(
                left.checked_add(require_operand(right)?)
                    .ok_or(CommandError::Overflow)?),
            Operator::Subtraction => self.valid_result(
                left.checked_sub(require_operand(right)?)
                    .ok_or(CommandError::Overflow)?),
            Operator::Power => {
                let right: Decimal = require_operand(right)?;

                // last index - 1 (pre-last) - 1 (leading zero)
                const DECIMAL_POINT_TO_ROUND_TO: u32 = (BUFFER_SIZE - 1 - 1) as u32; // default: 21

                let result: Decimal = if right.scale() == 0 {
                    left.checked_powd(right)
                        .ok_or(CommandError::Overflow)?
                } else {
                    let ln_left: Decimal = left.ln();
                    let tolerance = Decimal::ZERO;
                    right.checked_mul(ln_left)
                        .ok_or(CommandError::Overflow)?
                        .checked_exp_with_tolerance(tolerance)
                        .ok_or(CommandError::Overflow)?
                }
                .round_dp(DECIMAL_POINT_TO_ROUND_TO);

                self.valid_result(result)
            },
            Operator::NaturalLogarithm => self.valid_result(
                left.checked_ln()
                    .ok_or(CommandError::Overflow)?),
            Operator::Sine => self.valid_result(
                left.checked_sin()
                    .ok_or(CommandError::Overflow)?),
            Operator::Cosine => self.valid_result(
                left.checked_cos()
                    .ok_or(CommandError::Overflow)?),
        };
        result.map(|result| result.normalize())
    }

    fn valid_result(&self, result: Decimal) -> Result<Decimal, CommandError> {
        dbg!(result); ////
        if result.trunc().abs() > BUFFER_MAX_NUMBER_DEC {
            Err(CommandError::OutOfBufferRange(result))?
        }
        Ok(result)
    }

    pub fn is_unary(&self) -> bool {
        match self {
            Operator::Division => false,
            Operator::Multiplication => false,
            Operator::Addition => false,
            Operator::Subtraction => false,
            Operator::Power => false,
            Operator::NaturalLogarithm => true,
            Operator::Sine => true,
            Operator::Cosine => true,
        }
    }
}

fn require_operand(operand: Option<Decimal>) -> Result<Decimal, CommandError> {
    operand.ok_or(CommandError::OperandIsMissing)
}

impl TryFrom<char> for Operator {
    type Error = CommandError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '/' => Ok(Operator::Division),
            '*' => Ok(Operator::Multiplication),
            '+' => Ok(Operator::Addition),
            '-' => Ok(Operator::Subtraction),
            '^' => Ok(Operator::Power),
            _ => Err(CommandError::IncorrectOperation("Incorrect operation symbol".into()))?
        }
    }
}