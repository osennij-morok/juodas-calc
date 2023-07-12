use std::{
    cell::{Ref, RefCell}, 
    str::FromStr
};

use arboard::Clipboard;
use iced::{
    Application, 
    Theme, 
    executor, 
    widget::{button, text, column, container, Container, Column, row, Row, Space, Text}, 
    Command, 
    Settings, 
    keyboard::{
        self, KeyCode, Modifiers
    }, 
    subscription, 
    Event, 
    alignment::{self, Horizontal}, 
    Length, 
    window::{self, Icon}
};
use image::ImageFormat;
use rust_decimal::Decimal;

use crate::{
    calculator::{Calculator, operand::Operand, error::CommandError, self, operator::Operator}, 
    clipboard
};

#[cfg(test)]
mod tests;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    const WIDTH: u32 = 400; // 260
    const HEIGHT: u32 = 260;
    Ok(CalcState::run(Settings {
        window: window::Settings {
            size: (WIDTH, HEIGHT),
            min_size: Some((WIDTH, HEIGHT)),
            max_size: Some((WIDTH, HEIGHT)),
            //resizable: false,
            icon: Some(
                Icon::from_file_data(include_bytes!("../../calculator-48.png"), 
                Some(ImageFormat::Png))?),
            ..Default::default()
        },
        ..Default::default()
    })?)
}

#[derive(Debug)]
struct CalcState {
    calc: RefCell<Calculator>,
    line: RefCell<String>,
}

impl Default for CalcState {
    fn default() -> Self {
        Self { 
            calc: RefCell::new(Calculator::new()),
            // 'e' + 'M' + space + '-' + '.' + operand buffer
            line: RefCell::new(
                String::with_capacity(
                    1 + 1 + 1 + 1 + 1 + calculator::BUFFER_SIZE)), 
        }
    }
} 

#[derive(Debug, Clone, Copy)]
enum CalcMessage {
    Nothing,

    MRC,
    MMinus,
    MPlus,
    Clear,
    
    Power,
    NaturalLogarithm,
    Sine,
    Cosine,
    Pi,
    _Set(Decimal),
    Percentage,
    EulersNumber,
    Copy,
    Paste,

    Symbol(char),
}

impl Application for CalcState {
    type Executor = executor::Default;
    type Message = CalcMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let mut state = CalcState::default();
        state.line.get_mut().push('0');
        (state, Command::none())
    }

    fn title(&self) -> String {
        "Калькулятор".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {

            CalcMessage::Nothing => {}

            CalcMessage::MRC => {
                let calc = self.calc.get_mut();
                match calc.memory_mrc() {
                    Ok(_) => {},
                    Err(err) => {
                        dbg!(&err);
                    },
                }
                self.update_state_from_calc(Ok(()));
            },
            
            CalcMessage::MMinus => {
                let calc = self.calc.get_mut();
                match calc.memory_sub() {
                    Ok(_) => {},
                    Err(err) => {
                        dbg!(&err);
                    },
                }
            },

            CalcMessage::MPlus => {
                let calc = self.calc.get_mut();
                match calc.memory_add() {
                    Ok(_) => {},
                    Err(err) => {
                        dbg!(&err);
                    },
                }
            },

            CalcMessage::Clear => {
                let calc_ref: &mut Calculator = self.calc.get_mut();
                calc_ref.erase();
                self.update_state_from_calc(Ok(()));
            },

            CalcMessage::Symbol(ch) => {
                let calc_ref = self.calc.get_mut();
                let calc_response: Result<(), CommandError> = calc_ref.symbol_in(ch).map(|_| ());
                self.update_state_from_calc(calc_response);
            },

            CalcMessage::Power => {
                let calc_ref = self.calc.get_mut();
                let calc_response: Result<(), CommandError> = calc_ref.operator_in(Operator::Power).map(|_| ());
                self.update_state_from_calc(calc_response);
            },

            CalcMessage::NaturalLogarithm => {
                let calc_ref = self.calc.get_mut();
                let calc_response: Result<(), CommandError> = calc_ref.operator_in(Operator::NaturalLogarithm).map(|_| ());
                self.update_state_from_calc(calc_response);
            },

            CalcMessage::Sine => {
                let calc_ref = self.calc.get_mut();
                let calc_response: Result<(), CommandError> = calc_ref.operator_in(Operator::Sine).map(|_| ());
                self.update_state_from_calc(calc_response);
            },

            CalcMessage::Cosine => {
                let calc_ref = self.calc.get_mut();
                let calc_response: Result<(), CommandError> = calc_ref.operator_in(Operator::Cosine).map(|_| ());
                self.update_state_from_calc(calc_response);
            },

            CalcMessage::Pi => {
                let calc_ref = self.calc.get_mut();
                calc_ref.pi();
                self.update_state_from_calc(Ok(()));
            },

            CalcMessage::_Set(_) => {},

            CalcMessage::Percentage => {
                let calc_ref = self.calc.get_mut();
                let calc_response: Result<(), CommandError> = calc_ref.percentage();
                self.update_state_from_calc(calc_response);
            },

            CalcMessage::EulersNumber => {
                let calc_ref = self.calc.get_mut();
                calc_ref.eulers_number();
                self.update_state_from_calc(Ok(()));
            },

            CalcMessage::Copy => {
                let calc: Ref<Calculator> = self.calc.borrow();
                let current_operand_str: String = calc.current_operand_to_str();
                println!("STRING TO COPY: {}", current_operand_str);
                let mut clipboard_ref = clipboard::CLIPBOARD.lock().unwrap();
                let clipboard: &mut Clipboard = if let Some(clipboard) = clipboard_ref.as_mut() {
                    clipboard
                } else {
                    println!("ERROR: UNABLE TO ACCESS CLIPBOARD");
                    return Command::none()
                };
                if let Err(err) = clipboard.set_text(current_operand_str) {
                    println!("ERROR: {}", err.to_string());
                }
            },

            CalcMessage::Paste => {
                let calc: &mut Calculator = self.calc.get_mut();
                let mut clipboard_ref = clipboard::CLIPBOARD.lock().unwrap();
                let clipboard: &mut Clipboard = if let Some(clipboard) = clipboard_ref.as_mut() {
                    clipboard
                } else {
                    println!("ERROR: UNABLE TO ACCESS CLIPBOARD");
                    return Command::none()
                };
                let parsed_number: Decimal = if let Some(parsed_number) = clipboard
                    .get_text()
                    .ok()
                    .and_then(|clipboard_content| Decimal::from_str(&clipboard_content).ok()) 
                {
                    parsed_number
                } else {
                    return Command::none()
                };
                calc.set_current_operand(parsed_number);
                self.update_state_from_calc(Ok(()));
            },
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        // let col: Column<CalcMessage> = column![
        //     button("+").on_press(CalcMessage::Inc),
        //     text(self.number).size(50),
        //     button("-").on_press(CalcMessage::Dec)
        // ];
        let calc_txt: iced::Element<CalcMessage> = text(self.line.borrow())
            .size(40)
            .horizontal_alignment(Horizontal::Right)
            .width(Length::Fill)
            // .width(Length::Shrink)
            .into();
        // button(text("MRC"))
        //     .width(Length::FillPortion(1))
        //     .on_press(CalcMessage::MRC);
        // let t: Text<'_> = text("M-")
        //     .width(Length::Shrink)
        //     .horizontal_alignment(Horizontal::Center);
        let row_1: Row<CalcMessage> = row!(button(Self::btn_text("MRC"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::MRC),
                                           button(Self::btn_text("M-"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::MMinus),
                                           button(Self::btn_text("M+"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::MPlus),
                                           button(Self::btn_text("π"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Pi),
                                           button(Self::btn_text("e"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::EulersNumber),
                                           button(Self::btn_text("C"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Clear))
                                        .spacing(5);

        let row_2: Row<CalcMessage> = row!(button(Self::btn_text("7"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('7')),   
                                           button(Self::btn_text("8"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('8')),  
                                           button(Self::btn_text("9"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('9')),  
                                           button(Self::btn_text("×"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('*')), 
                                           button(Self::btn_text("÷"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('/')), 
                                           button(Self::btn_text("sin"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Sine))
                                        .spacing(5);

        let row_3: Row<CalcMessage> = row!(button(Self::btn_text("4"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('4')),  
                                           button(Self::btn_text("5"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('5')),  
                                           button(Self::btn_text("6"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('6')),  
                                        //    Space::new(Length::FillPortion(1), Length::Shrink),
                                           button(Self::btn_text("+"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('+')), 
                                           button(Self::btn_text("-"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('-')), 
                                           button(Self::btn_text("cos"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Cosine))
                                        .spacing(5);

        let row_4: Row<CalcMessage> = row!(button(Self::btn_text("1"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('1')),  
                                           button(Self::btn_text("2"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('2')),  
                                           button(Self::btn_text("3"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('3')),  
                                           button(Self::btn_text("a^n"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Power),
                                           button(Self::btn_text("ln"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::NaturalLogarithm), 
                                           button(Self::btn_text("%"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Percentage))
                                        .spacing(5);

        let row_5: Row<CalcMessage> = row!(button(Self::btn_text("0"))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('0')),  
                                           Space::new(Length::FillPortion(1), Length::Shrink),
                                           button(Self::btn_text("."))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('.')),               
                                           Space::new(Length::FillPortion(1), Length::Shrink),
                                           Space::new(Length::FillPortion(1), Length::Shrink),
                                           button(Self::btn_text("="))
                                               .width(Length::FillPortion(1))
                                               .on_press(CalcMessage::Symbol('=')))
                                        .spacing(5);
        let main_col: Column<CalcMessage> = column!(
            calc_txt,
            row_1,
            row_2,
            row_3,
            row_4,
            row_5
        );
        let main_col = main_col
            .spacing(5)
            .height(Length::Shrink);
        let c: Container<CalcMessage> = container(main_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(alignment::Vertical::Bottom)
            .center_x()
            .center_y()
            .padding(10);
        c.into()
    }


    fn subscription(&self) -> iced::Subscription<Self::Message> {
        subscription::events().map(Self::on_event)
    }
}

impl CalcState {
    fn on_event(event: Event) -> CalcMessage {
        use CalcMessage::*;
        match event {

            // Event::PlatformSpecific(_) => {
            //     //println("Platform specific event");
            //     Nothing
            // },

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::D, 
                modifiers: _ 
            }) => Nothing,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::A, 
                modifiers: _ 
            }) => Nothing,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::C, 
                modifiers: Modifiers::CTRL 
            }) => Copy,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::C, 
                modifiers: _ 
            }) => Cosine,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::E, 
                modifiers: _ 
            }) => EulersNumber,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::L, 
                modifiers: _ 
            }) => NaturalLogarithm,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::P, 
                modifiers: _ 
            }) => Pi,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::S, 
                modifiers: _ 
            }) => Sine,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::V, 
                modifiers: Modifiers::CTRL 
            }) => Paste,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::Key5, 
                modifiers: Modifiers::SHIFT // percentage
            }) => Percentage,

            Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: KeyCode::Backspace, 
                modifiers: _ 
            }) => Clear,

            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::Enter | KeyCode::NumpadEnter, 
                modifiers: _ 
            }) => Symbol('='),

            Event::Keyboard(keyboard::Event::CharacterReceived(ch)) => {
                if ch == '%' {
                    return Percentage
                }
                const COMMAND_CHARS: &[char] = &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 
                                                                '^', '=', '.', '-', '+', '*', '/'];
                if COMMAND_CHARS.contains(&ch) {
                    Symbol(ch)
                } else {
                    Nothing
                }
            },

            Event::Keyboard(_) => Nothing,
            Event::Mouse(_) => Nothing,
            Event::Window(_) => Nothing,
            Event::Touch(_) => Nothing,
        }
    }

    fn btn_text(content: &str) -> Text<'_> {
        text(content)
            .width(Length::Shrink)
            .horizontal_alignment(Horizontal::Center)
    }

    // fn update_from_calc_state(&mut self, calc_response: Result<(), CommandError>) {
    fn update_state_from_calc(&mut self, calc_response: Result<(), CommandError>) {
        if calc_response.is_ok() {
            let calc_ref: Ref<Calculator> = self.calc.borrow();
            let calc_ref: &Calculator = &calc_ref;
            use calculator::State::*;
            match &calc_ref.state {
                ReadingLeftOrOperator(operand_cell) => {
                    self.write_operand_from_cell(operand_cell);
                },
                ReadingRight { left: operand_cell, operator: _ } => {
                    self.write_operand_from_cell(operand_cell);
                },
                ReadingRightOrNextAction {
                    left: _, 
                    operator: _, 
                    right: operand_cell 
                } => {
                    self.write_operand_from_cell(operand_cell);
                },
                Result(result_cell) => {
                    self.write_operand_from_cell(result_cell);
                },
            }
            return;
        }
        self.line.get_mut().clear();
        let line: &mut String = self.line.get_mut();
        line.push_str("e    "); // for e + M + space + '-' + '.'
        self.calc.get_mut().erase_all();
        for _ in 0..calculator::BUFFER_SIZE - 1 {
            line.push(' ');
        }
        line.push('0');
    }

    fn write_operand(&self, operand: &Operand) {
        let line: &mut String = &mut self.line.borrow_mut();
        if operand.is_positive() {
            line.push(' '); // for '-'
        }
        if !operand.has_dot() {
            line.push(' '); // for '.'
        }
        let free_cells_count: usize = calculator::BUFFER_SIZE - operand.numbers_count();
        for _ in 0..free_cells_count {
            line.push(' ');
        }
        dbg!(&operand); ////
        line.push_str(&operand.current_state_string());
    }

    fn write_operand_from_cell(&self, cell: &RefCell<Option<Operand>>) {
        let operand_option: Ref<Option<Operand>> = cell.borrow();
        let operand: &Operand = operand_option.as_ref().unwrap();
        // let line: &mut String = &mut self.line.borrow_mut();
        self.line.borrow_mut().clear();
        self.line.borrow_mut().push_str("   "); // for e + M + space
        self.write_operand(operand);
    }
}
