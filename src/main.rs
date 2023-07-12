#![windows_subsystem = "windows"]

#[cfg(test)]
mod tests;
mod calculator;
mod gui;
mod clipboard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gui::run()
}