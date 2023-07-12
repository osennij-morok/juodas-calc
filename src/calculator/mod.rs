use std::cell::RefCell;

use rust_decimal::Decimal;

use rust_decimal_macros::dec;

use self::{operand::Operand, operator::Operator, error::CommandError};

#[cfg(test)]
mod tests;
pub mod operand;
pub mod operator;
pub mod error;

pub const BUFFER_SIZE: usize = 16; // 8 
// these both must be consistent
const BUFFER_MAX_NUMBER_DEC: Decimal = dec!(9999_9999_9999_9999.0);
// these both must be consistent
const BUFFER_MAX_NUMBER: f64 = 9999_9999.0;
const DOT_SYMBOL: char = '.';

#[derive(Debug)]
pub struct Calculator {
    pub state: State,
    memory: Memory
}

pub type OperandCell = RefCell<Option<Operand>>;

#[derive(Debug)]
pub enum State {
    ReadingLeftOrOperator(OperandCell),
    ReadingRight {
        left: OperandCell,
        operator: Operator,
    },
    ReadingRightOrNextAction {
        left: OperandCell,
        operator: Operator,
        right: OperandCell
    },
    Result(OperandCell)
}

impl State {
    pub fn begin() -> Self {
        Self::ReadingLeftOrOperator(cell_with_operand(Operand::new(), None))
    }
}

#[derive(Debug, Default)]
struct Memory {
    value: Decimal
}

impl Calculator {

    pub fn new() -> Self {
        Self::with_state(State::begin())
    }

    fn with_state(state: State) -> Self {
        Self { 
            state, 
            memory: Default::default() 
        }
    }

    pub fn symbol_in(&mut self, symbol: char) -> Result<&mut Self, CommandError> {
        if is_operation(symbol) {
            return self.operator_in(Operator::try_from(symbol)?)
        }
        match &mut self.state {
            State::ReadingLeftOrOperator(operand_cell) => {
                let mut operand: Operand = operand_cell.take().unwrap();
                let operand_should_be_replaced: bool = operand.should_reset_on_clear();
                if operand_should_be_replaced {
                    operand.send_erase();
                }
                operand.send_symbol(symbol);
                let reset_operand_on_erase = Some(false);
                self.state = State::ReadingLeftOrOperator(
                    cell_with_operand(operand, reset_operand_on_erase));
                Ok(self)
            },
            State::ReadingRight { left, operator } => {
                let mut right = Operand::new();
                // maybe, it is worth to fix with checking what symbol is sent
                right.send_symbol(symbol);
                let reset_operand_on_erase = Some(false);
                self.state = State::ReadingRightOrNextAction { 
                    left: cell_move(left), 
                    operator: *operator, 
                    right: cell_with_operand(right, reset_operand_on_erase) 
                };
                Ok(self)
            },
            State::ReadingRightOrNextAction { 
                left, 
                operator, 
                right 
            } => {
                // await /, *, +, - or =
                let left: Operand = left.take().unwrap();
                let mut right: Operand = right.take().unwrap();
                if is_number(symbol) || symbol == '.' {
                    let operand_should_be_replaced: bool = right.should_reset_on_clear();
                    if operand_should_be_replaced {
                        right.send_erase();
                    }
                    right.send_symbol(symbol);
                    let reset_operand_on_erase = Some(false);
                    self.state = State::ReadingRightOrNextAction { 
                        left: cell_with_operand(left, None), 
                        operator: *operator, 
                        right: cell_with_operand(right, reset_operand_on_erase) 
                    };
                    return Ok(self)
                }
                if is_eq(symbol) {
                    let left: Decimal = left.try_into()?;
                    let right: Decimal = right.try_into()?;
                    let result: Decimal = operator.apply(left, Some(right))?;
                    let result: Operand = Operand::try_from(result)?;
                    let reset_operand_on_erase = Some(true);
                    self.state = State::Result(
                        cell_with_operand(result, reset_operand_on_erase));
                    return Ok(self)
                }
                self.state = State::ReadingRightOrNextAction { 
                    left: cell_with_operand(left, None), 
                    operator: *operator, 
                    right: cell_with_operand(right, None) 
                };
                Ok(self)
            },
            State::Result(_) => {
                self.state = State::begin();
                self.symbol_in(symbol)
            }
        }
    }

    pub fn operator_in(&mut self, operator: Operator) -> Result<&mut Self, CommandError> {
        match &mut self.state {
            State::ReadingLeftOrOperator(operand_cell) => {
                let operand: Operand = operand_cell.take().unwrap();
                let reset_operand_on_erase = Some(true);
                if operator.is_unary() {
                    let result: Decimal = operator.apply(operand.try_into()?, None)?;
                    let result: Operand = Operand::try_from(result)?;
                    self.state = State::Result(cell_with_operand(result, reset_operand_on_erase));
                    return Ok(self)
                }
                self.state = State::ReadingRight { 
                    left: cell_with_operand(operand, None), 
                    operator 
                };
                return Ok(self)
            },
            State::ReadingRight { 
                left: operand_cell, 
                operator: _ 
            } => {
                if operator.is_unary() {
                    let operand: Operand = operand_cell.take().unwrap();
                    let operand: Decimal = operand.try_into()?;
                    let result: Decimal = operator.apply(operand, None)?;
                    let result = Operand::try_from(result)?;
                    let reset_operand_on_erase = Some(true);
                    self.state = State::Result(
                        cell_with_operand(result, reset_operand_on_erase));
                    return Ok(self)
                }
                self.state = State::ReadingLeftOrOperator(cell_move(operand_cell));
                self.operator_in(operator)
            },
            State::ReadingRightOrNextAction { 
                left, 
                operator: first_operator, 
                right 
            } => {
                let right: Operand = right.take().unwrap();
                let right: Decimal = right.try_into()?;
                if operator.is_unary() {
                    let result: Decimal = operator.apply(right, None)?;
                    let result: Operand = Operand::try_from(result)?;
                    let first_operator: Operator = *first_operator;
                    let reset_operand_on_erase = Some(true);
                    self.state = State::ReadingRightOrNextAction { 
                        left: cell_move(left), 
                        operator: first_operator, 
                        right: cell_with_operand(result, reset_operand_on_erase)
                    };
                    return Ok(self)
                }
                let left: Operand = left.take().unwrap();
                let left: Decimal = left.try_into()?;
                let result: Decimal = first_operator.apply(left, Some(right))?;
                let result: Operand = Operand::try_from(result)?;
                self.state = State::ReadingRight { 
                    left: cell_with_operand(result, None), 
                    operator 
                };
                Ok(self)
            },
            State::Result(operand_cell) => {
                let operand: Operand = operand_cell.take().unwrap();
                if operator.is_unary() {
                    let operand: Decimal = operand.try_into()?;
                    let result: Decimal = operator.apply(operand, None)?;
                    let result: Operand = Operand::try_from(result)?;
                    let reset_operand_on_erase = Some(true);
                    self.state = State::Result(
                        cell_with_operand(result, reset_operand_on_erase));
                    return Ok(self)
                }
                self.state = State::ReadingRight { 
                    left: cell_with_operand(operand, None), 
                    operator
                };
                Ok(self)
            },
        }
    }

    fn current_operand_to_dec(&self) -> Decimal {
        match &self.state {
            State::ReadingLeftOrOperator(operand_cell) 
                => cell_ref_to_dec(operand_cell),
            State::ReadingRight { 
                left: operand_cell, 
                operator: _ 
            } => cell_ref_to_dec(operand_cell),
            State::ReadingRightOrNextAction { 
                left: _, 
                operator: _, 
                right: operand_cell 
            } => cell_ref_to_dec(operand_cell),
            State::Result(operand_cell)
                => cell_ref_to_dec(operand_cell),
        }
    }

    pub fn current_operand_to_str(&self) -> String {
        match &self.state {
            State::ReadingLeftOrOperator(operand_cell) 
                => cell_ref_to_str(operand_cell),
            State::ReadingRight { 
                left: operand_cell, 
                operator: _ 
            } => cell_ref_to_str(operand_cell),
            State::ReadingRightOrNextAction { 
                left: _, 
                operator: _, 
                right: operand_cell 
            } => cell_ref_to_str(operand_cell),
            State::Result(operand_cell)
                => cell_ref_to_str(operand_cell),
        }
    }

    pub fn set_current_operand(&mut self, value: Decimal) {
        let operand = Operand::try_from(value).unwrap(); ////
        let reset_operand_on_erase = Some(true);
        let new_cell: OperandCell = cell_with_operand(operand, reset_operand_on_erase);
        match &mut self.state {
            State::ReadingLeftOrOperator(_) 
                => self.state = State::ReadingLeftOrOperator(new_cell),
            State::ReadingRight { 
                left, 
                operator 
            } => self.state = State::ReadingRightOrNextAction { 
                left: cell_move(left), 
                operator: *operator,
                right: new_cell
            },
            State::ReadingRightOrNextAction { 
                left, 
                operator, 
                right: _ 
            } => self.state = State::ReadingRightOrNextAction { 
                left: cell_move(left), 
                operator: *operator, 
                right: new_cell 
            },
            State::Result(_)
                => self.state = State::Result(new_cell),
        }
        ;
    }

    pub fn memory_add(&mut self) -> Result<(), CommandError> {
        self.memory_apply(Operator::Addition)
    }

    pub fn memory_sub(&mut self) -> Result<(), CommandError> {
        self.memory_apply(Operator::Subtraction)
    }

    fn memory_apply(&mut self, operator: Operator) -> Result<(), CommandError> {
        let current_operand_value: Decimal = self.current_operand_to_dec();
        self.memory.value = operator.apply(current_operand_value, Some(self.memory.value))?;
        Ok(())
    }

    pub fn memory_mrc(&mut self) -> Result<(), CommandError> {
        let current_operand_value: Decimal = self.current_operand_to_dec();
        if current_operand_value == self.memory.value {
            self.memory.value = Decimal::ZERO;
            return Ok(())
        }
        self.set_current_operand(self.memory.value);
        Ok(())
    }

    pub fn percentage(&mut self) -> Result<(), CommandError> {
        match &self.state {
            State::ReadingLeftOrOperator(_) => {},
            State::ReadingRight { .. } => {},
            State::ReadingRightOrNextAction { 
                left, 
                operator, 
                right 
            } => {
                let left: Operand = left.take().unwrap();
                let right: Operand = right.take().unwrap();
                let left_dec: Decimal = left.try_into()?;
                let right_dec: Decimal = right.try_into()?;
                match operator {
                    // Operator::Division => todo!(),
                    Operator::Addition => {
                        let percent: Decimal = checked_percent(left_dec, right_dec)?;
                        let result: Decimal = left_dec.checked_add(percent)
                            .ok_or(CommandError::Overflow)?
                            .normalize();
                        let result: Operand = Operand::try_from(result)?;
                        let reset_operand_on_erase = Some(true);
                        self.state = State::Result(
                            cell_with_operand(result, reset_operand_on_erase))
                    },
                    Operator::Multiplication => {
                        let percent: Decimal = checked_percent(left_dec, right_dec)?.normalize();
                        let percent: Operand = Operand::try_from(percent)?;
                        let reset_operand_on_erase = Some(true);
                        self.state = State::Result(
                            cell_with_operand(percent, reset_operand_on_erase))
                    },
                    Operator::Subtraction => {
                        let percent: Decimal = checked_percent(left_dec, right_dec)?;
                        let result: Decimal = left_dec.checked_sub(percent)
                            .ok_or(CommandError::Overflow)?
                            .normalize();
                        let result: Operand = Operand::try_from(result)?;
                        let reset_operand_on_erase = Some(true);
                        self.state = State::Result(
                            cell_with_operand(result, reset_operand_on_erase))
                    },
                    _ => {}
                }
            },
            State::Result(_) => {},
        }
        Ok(())
    }

    pub fn pi(&mut self) {
        self.set_current_operand(Decimal::PI);
    }

    pub fn eulers_number(&mut self) {
        self.set_current_operand(Decimal::E);
    }

    pub fn erase_all(&mut self) -> &mut Self {
        self.state = State::begin();
        self
    }

    pub fn erase(&mut self) -> &mut Self {
        match &mut self.state {
            State::ReadingLeftOrOperator(operand_cell) => {
                let mut operand: Operand = operand_cell.take().unwrap();
                operand.send_erase();
                operand_cell.replace(Some(operand));
                self
            },
            State::ReadingRight { .. } => self.erase_all(),
            State::ReadingRightOrNextAction { right: right_cell, .. } => {
                let mut operand: Operand = right_cell.take().unwrap();
                operand.send_erase();
                right_cell.replace(Some(operand));
                self
            },
            State::Result(_) => {
                self.state = State::begin();
                self
            }
        }
    }
}

fn checked_percent(number: Decimal, percent: Decimal) -> Result<Decimal, CommandError> {
    (number / dec!(100))
        .checked_mul(percent)
        .ok_or(CommandError::Overflow)
}

fn cell_with_operand(mut operand: Operand, reset_operand_on_erase: Option<bool>) -> OperandCell {
    if let Some(reset_operand_on_erase) = reset_operand_on_erase {
        operand.set_should_reset_on_clear(reset_operand_on_erase);
    }
    RefCell::new(Some(operand))
}

fn cell_move(cell: &mut OperandCell) -> OperandCell {
    let operand: Operand = cell.take().unwrap();
    RefCell::new(Some(operand))
}

fn cell_ref_to_dec(cell_ref: &OperandCell) -> Decimal {
    cell_ref.borrow()
        .as_ref()
        .map(|operand| operand.try_into())
        .unwrap()
        .unwrap()
}

fn cell_ref_to_str(cell_ref: &OperandCell) -> String {
    cell_ref.borrow()
        .as_ref()
        .map(|operand| operand.to_string())
        .unwrap()
}

fn is_operation(symbol: char) -> bool {
    const OPERATIONS: &[char] = &['/', '+', '-', '*', '^'];
    OPERATIONS.contains(&symbol)
}

fn is_number(symbol: char) -> bool {
    const NUMBERS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    NUMBERS.contains(&symbol)
}

fn is_eq(symbol: char) -> bool {
    symbol == '='
}
