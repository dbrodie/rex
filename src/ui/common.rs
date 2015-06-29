use std::char;

signal_decl!{Canceled(Option<String>)}

pub struct Rect<T> {
    pub top: T,
    pub left: T,
    pub bottom: T,
    pub right: T
}

pub fn u4_to_hex(b: u8) -> char {
    char::from_digit(b as u32, 16).unwrap()
}
pub fn u8_to_hex(b: u8) -> (char, char) {
    (u4_to_hex((b >> 4) & 0xF), u4_to_hex(b & 0xF))
}
