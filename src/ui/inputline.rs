use std::str;
use rustc_serialize::hex::FromHex;
use std::char;
use std::path::PathBuf;
use util::string_with_repeat;
use rustbox::{RustBox, Color, RB_NORMAL, RB_BOLD};
use rustbox::keyboard::Key;

use super::super::buffer::Buffer;
use super::super::segment::Segment;
use super::super::signals;
use super::RustBoxEx::{RustBoxEx, Style};
use super::input::Input;


use super::common::{Rect, Canceled};

pub enum BaseInputLineActions {
    Edit(char),
    Ctrl(char),
    MoveLeft,
    MoveRight,
    DeleteWithMove,
    Ok,
    Cancel
}

pub trait InputLine {
    fn input(&mut self, input: &Input, key: Key) -> bool;
    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool);
}

struct BaseInputLine {
    prefix: String,
    data: Vec<u8>,
    input_pos: isize,
}

impl BaseInputLine {
    fn new(prefix: String) -> BaseInputLine {
        BaseInputLine {
            prefix: prefix,
            data: vec!(),
            input_pos: 0,
        }
    }
}

impl InputLine for BaseInputLine {
    fn input(&mut self, input: &Input, key: Key) -> bool {
        let action = if let Some(action) = input.inputline_input(key) { action } else {
            return false;
        };

        match action {
            BaseInputLineActions::MoveLeft => {
                if self.input_pos > 0 {
                    self.input_pos -= 1;
                }
            }
            BaseInputLineActions::MoveRight => {
                if self.input_pos < self.data.len() as isize {
                    self.input_pos += 1;
                }
            }
            BaseInputLineActions::Edit(ch) => {
                if ch.len_utf8() == 1 && ch.is_alphanumeric() {
                    self.data.insert(self.input_pos as usize, ch as u8);
                    self.input_pos += 1;
                } else {
                    // TODO: MAke it printable rather than alphanumeric
                }
            }
            BaseInputLineActions::DeleteWithMove => {
                if self.input_pos > 0 {
                    self.input_pos -= 1;
                    self.data.remove(self.input_pos as usize);
                }
            }
            _ => return false
        };

        return true;
    }

    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool) {
        rb.print_style(area.left as usize, area.top as usize, Style::InputLine,
                 &string_with_repeat(' ', (area.right - area.left) as usize));
        rb.print_style(area.left as usize, area.top as usize, Style::InputLine,
                 &format!("{}{}", self.prefix, str::from_utf8(&self.data).unwrap()));
        if has_focus {
            rb.set_cursor(self.prefix.len() as isize + self.input_pos, (area.top as isize));
        }
    }
}

enum RadixType {
    DecRadix,
    HexRadix,
    OctRadix,
}

signal_decl!{GotoEvent(isize)}

pub struct GotoInputLine {
    base: BaseInputLine,
    radix: RadixType,
    pub on_done: GotoEvent,
    pub on_cancel: Canceled,
}

impl GotoInputLine {
    pub fn new() -> GotoInputLine {
        GotoInputLine {
            base: BaseInputLine::new("Goto (Dec):".to_string()),
            radix: RadixType::DecRadix,
            on_done: Default::default(),
            on_cancel: Default::default(),
        }
    }

    fn set_radix(&mut self, r: RadixType) {
        self.radix = r;
        self.base.prefix = match self.radix {
            RadixType::DecRadix => "Goto (Dec):".to_string(),
            RadixType::HexRadix => "Goto (Hex):".to_string(),
            RadixType::OctRadix => "Goto (Oct):".to_string(),
        }
    }

    fn do_goto(&mut self) {
        let radix = match self.radix {
            RadixType::DecRadix => 10,
            RadixType::HexRadix => 16,
            RadixType::OctRadix => 8,
        };

        let pos: Option<isize> = match str::from_utf8(&self.base.data) {
            Ok(gs) => isize::from_str_radix(&gs, radix).ok(),
            Err(_) => None
        };

        match pos {
            Some(pos) => {
                self.on_done.signal(pos)
            }
            None => {
                self.on_cancel.signal(Some(format!("Bad position!")));
            }
        };

    }
}

impl InputLine for GotoInputLine {
    fn input(&mut self, input: &Input, key: Key) -> bool {
        if self.base.input(input, key) { return true }

        let action = if let Some(action) = input.inputline_input(key) { action } else {
            return false;
        };

        match action {
            BaseInputLineActions::Ok => {
                self.do_goto();
                // self.done_state = Some(true);
                true
            }
            BaseInputLineActions::Cancel => {
                self.on_cancel.signal(None);
                // self.done_state = Some(false);
                true
            }

            BaseInputLineActions::Ctrl('d') => {
                self.set_radix(RadixType::DecRadix);
                true
            }
            BaseInputLineActions::Ctrl('h') => {
                self.set_radix(RadixType::HexRadix);
                true
            }
            BaseInputLineActions::Ctrl('o') => {
                self.set_radix(RadixType::OctRadix);
                true
            }

            _ => false
        }
    }

    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool) {
        self.base.draw(rb, area, has_focus)
    }
}

enum DataType {
    AsciiStr,
    UnicodeStr,
    HexStr,
}

signal_decl!{FindEvent(Vec<u8>)}

pub struct FindInputLine {
    base: BaseInputLine,
    data_type: DataType,
    pub on_find: FindEvent,
    pub on_cancel: Canceled,
}

impl FindInputLine {
    pub fn new() -> FindInputLine {
        FindInputLine {
            base: BaseInputLine::new("Find(Ascii): ".to_string()),
            data_type: DataType::AsciiStr,
            on_find: Default::default(),
            on_cancel: Default::default(),
        }
    }

    fn set_search_data_type(&mut self, dt: DataType) {
        self.data_type = dt;
        self.base.prefix = match self.data_type {
            DataType::AsciiStr => "Find(Ascii): ".to_string(),
            DataType::UnicodeStr => "Find(Uni): ".to_string(),
            DataType::HexStr => "Find(Hex): ".to_string(),
        }
    }

    fn do_find(&mut self) {
        let ll = str::from_utf8(&self.base.data).unwrap().from_hex();

        let needle: Vec<u8> = match self.data_type {
            DataType::AsciiStr => self.base.data.clone().into(),
            DataType::UnicodeStr => self.base.data.clone().into(),
            DataType::HexStr => {
                match ll {
                    Ok(n) => n,
                    Err(_) => {
                        self.on_cancel.signal(Some(format!("Bad hex value")));
                        return;
                    }
                }
            }
        };

        self.on_find.signal(needle);
    }
}

impl InputLine for FindInputLine {
    fn input(&mut self, input: &Input, key: Key) -> bool {
        if self.base.input(input, key) { return true }

        let action = if let Some(action) = input.inputline_input(key) { action } else {
            return false;
        };

        match action {
            BaseInputLineActions::Ok => {
                self.do_find();
                // self.done_state = Some(true);
                true
            }
            BaseInputLineActions::Cancel => {
                // self.done_state = Some(false);
                self.on_cancel.signal(None);
                true
            }

            BaseInputLineActions::Ctrl('a') => {
                self.set_search_data_type(DataType::AsciiStr);
                true
            }
            BaseInputLineActions::Ctrl('u') => {
                self.set_search_data_type(DataType::UnicodeStr);
                true
            }
            BaseInputLineActions::Ctrl('h') => {
                self.set_search_data_type(DataType::HexStr);
                true
            }

            _ => false
        }
    }

    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool) {
        self.base.draw(rb, area, has_focus)
    }
}

signal_decl!{PathEvent(PathBuf)}

pub struct PathInputLine {
    base: BaseInputLine,
    pub on_done: PathEvent,
    pub on_cancel: Canceled,
}

impl PathInputLine {
    pub fn new(prefix: String) -> PathInputLine {
        PathInputLine {
            base: BaseInputLine::new(prefix),
            on_done: Default::default(),
            on_cancel: Default::default()
        }
    }
}

impl InputLine for PathInputLine {
    fn input(&mut self, input: &Input, key: Key) -> bool {
        if self.base.input(input, key) { return true }

        let action = if let Some(action) = input.inputline_input(key) { action } else {
            return false;
        };

        match action {
            BaseInputLineActions::Ok => {
                self.on_done.signal(PathBuf::from(str::from_utf8(&self.base.data).unwrap()));
                true
            }
            BaseInputLineActions::Cancel => {
                self.on_cancel.signal(None);
                true
            }
            _ => false
        }
    }

    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool) {
        self.base.draw(rb, area, has_focus)
    }
}
