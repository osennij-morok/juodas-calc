use std::num::ParseFloatError;

use rust_decimal::Decimal;

use super::operator::Operator;

#[derive(Debug)]
pub enum CommandError {
    OperationFailure(Operator, String),
    OutOfBufferRange(Decimal),
    IncorrectOperation(String),
    IncorrectOperand(Decimal),
    OperandIsMissing,
    ParsingFailure {
        err: Option<ParseFloatError>
    },
    Overflow
}

impl ToString for CommandError {
    fn to_string(&self) -> String {
        use CommandError::*;
        match self {
            OperationFailure(_, err_msg) => err_msg.into(),
            OutOfBufferRange(value) => format!("Number {} is out of buffer range", value),
            IncorrectOperation(s) => s.into(),
            IncorrectOperand(operand) => format!("Incorrect operand: {}", operand),
            ParsingFailure { err } => if let Some(err) = err {
                err.to_string()
            } else {
                "".into()
            },
            OperandIsMissing => "Second operand is missing".into(),
            Overflow => "Overflow".into(),
        }
    }
}

impl From<CommandError> for Box<dyn std::error::Error> {
    fn from(err: CommandError) -> Self {
        err.to_string().into()
    }
}
