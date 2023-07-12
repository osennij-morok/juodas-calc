use std::sync::Mutex;

use arboard::Clipboard;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(Clipboard::new().ok());
}
