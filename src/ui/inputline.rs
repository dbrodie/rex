use std::str;
use rustc_serialize::hex::FromHex;
use std::path::PathBuf;

use rex_utils;
use rex_utils::rect::Rect;
use super::super::frontend::{Frontend, Style, KeyPress};
use super::input::Input;
use super::widget::Widget;


use super::common::Canceled;

pub enum BaseInputLineActions {
    Edit(char),
    Ctrl(char),
    MoveLeft,
    MoveRight,
    DeleteWithMove,
    Ok,
    Cancel
}

pub trait InputLineBehavior {
    fn get_prefix(&mut self) -> String;
    fn do_enter(&mut self, data: &[u8]);
    fn do_cancel(&mut self);
    fn do_shortcut(&mut self, shortcut: char) {

    }
}

pub struct InputLine<T:InputLineBehavior> {
    behavior: T,
    data: Vec<u8>,
    input_pos: isize,
}

impl<T:InputLineBehavior> InputLine<T> {
    pub fn new(behavior: T) -> InputLine<T> {
        InputLine {
            behavior: behavior,
            data: vec![],
            input_pos: 0,
        }
    }
}

impl<T:InputLineBehavior> Widget for InputLine<T> {
    fn input(&mut self, input: &Input, key: KeyPress) -> bool {
        let action = if let Some(action) = input.inputline_input(key) { action } else {
            return false;
        };

        match action {
            BaseInputLineActions::Ok => {
                self.behavior.do_enter(&self.data)
            }
            BaseInputLineActions::Cancel => {
                self.behavior.do_cancel()
            }
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
                if ch.len_utf8() == 1 {
                    self.data.insert(self.input_pos as usize, ch as u8);
                    self.input_pos += 1;
                } else {
                    // TODO: Make it printable rather than alphanumeric
                }
            }
            BaseInputLineActions::Ctrl(ch) => {
                self.behavior.do_shortcut(ch)
            }
            BaseInputLineActions::DeleteWithMove => {
                if self.input_pos > 0 {
                    self.input_pos -= 1;
                    self.data.remove(self.input_pos as usize);
                }
            }
        };

        return true;
    }

    fn draw(&mut self, rb: &mut Frontend, area: Rect<isize>, has_focus: bool) {
        let prefix = self.behavior.get_prefix();
        rb.print_style(area.left as usize, area.top as usize, Style::InputLine,
                 &rex_utils::string_with_repeat(' ', area.width as usize));
        rb.print_style(area.left as usize, area.top as usize, Style::InputLine,
                 &format!("{}{}", prefix, str::from_utf8(&self.data).unwrap()));
        if has_focus {
            rb.set_cursor(prefix.len() as isize + self.input_pos, (area.top as isize));
        }
    }
}

enum RadixType {
    DecRadix,
    HexRadix,
    OctRadix,
}

signal_decl!{GotoEvent(isize)}

pub struct GotoInputLineBehavior {
    radix: RadixType,
    pub on_done: GotoEvent,
    pub on_cancel: Canceled,
}

impl GotoInputLineBehavior {
    pub fn new() -> GotoInputLineBehavior {
        GotoInputLineBehavior {
            radix: RadixType::DecRadix,
            on_done: Default::default(),
            on_cancel: Default::default(),
        }
    }

    fn set_radix(&mut self, r: RadixType) {
        self.radix = r;
    }

    fn do_goto(&mut self, data: &[u8]) {
        let radix = match self.radix {
            RadixType::DecRadix => 10,
            RadixType::HexRadix => 16,
            RadixType::OctRadix => 8,
        };

        let pos: Option<isize> = match str::from_utf8(&data) {
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

impl InputLineBehavior for GotoInputLineBehavior {
    fn get_prefix(&mut self) -> String {
        match self.radix {
            RadixType::DecRadix => "Goto (Dec):".to_string(),
            RadixType::HexRadix => "Goto (Hex):".to_string(),
            RadixType::OctRadix => "Goto (Oct):".to_string(),
        }
    }

    fn do_enter(&mut self, data: &[u8]) {
        self.do_goto(data);
    }

    fn do_cancel(&mut self) {
        self.on_cancel.signal(None);
    }

    fn do_shortcut(&mut self, shortcut: char) {
        match shortcut {
            'd' => {
                self.set_radix(RadixType::DecRadix);
            }
            'h' => {
                self.set_radix(RadixType::HexRadix);
            }
            'o' => {
                self.set_radix(RadixType::OctRadix);
            }
            _ => ()
        }
    }
}

enum DataType {
    AsciiStr,
    UnicodeStr,
    HexStr,
}

signal_decl!{FindEvent(Vec<u8>)}

pub struct FindInputLine {
    data_type: DataType,
    pub on_find: FindEvent,
    pub on_cancel: Canceled,
}

impl FindInputLine {
    pub fn new() -> FindInputLine {
        FindInputLine {
            data_type: DataType::AsciiStr,
            on_find: Default::default(),
            on_cancel: Default::default(),
        }
    }

    fn set_search_data_type(&mut self, dt: DataType) {
        self.data_type = dt;
    }

    fn do_find(&mut self, data: &[u8]) {
        let ll = str::from_utf8(data).unwrap().from_hex();

        let needle: Vec<u8> = match self.data_type {
            DataType::AsciiStr => data.clone().into(),
            DataType::UnicodeStr => data.clone().into(),
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

impl InputLineBehavior for FindInputLine {
    fn get_prefix(&mut self) -> String {
        match self.data_type {
            DataType::AsciiStr => "Find(Ascii): ".to_string(),
            DataType::UnicodeStr => "Find(Uni): ".to_string(),
            DataType::HexStr => "Find(Hex): ".to_string(),
        }
    }

    fn do_enter(&mut self, data: &[u8]) {
        self.do_find(data);
    }

    fn do_cancel(&mut self) {
        self.on_cancel.signal(None);
    }

    fn do_shortcut(&mut self, shortcut: char) {
        match shortcut {
            'a' => {
                self.set_search_data_type(DataType::AsciiStr);
            }
            'u' => {
                self.set_search_data_type(DataType::UnicodeStr);
            }
            'h' => {
                self.set_search_data_type(DataType::HexStr);
            }
            _ => ()
        }
    }
}

signal_decl!{PathEvent(PathBuf)}

pub struct PathInputLine {
    pub on_done: PathEvent,
    pub on_cancel: Canceled,
    prefix: String,
}

impl PathInputLine {
    pub fn new(prefix: String) -> PathInputLine {
        PathInputLine {
            prefix: prefix,
            on_done: Default::default(),
            on_cancel: Default::default()
        }
    }
}

impl InputLineBehavior for PathInputLine {
    fn get_prefix(&mut self) -> String {
        self.prefix.clone()
    }

    fn do_enter(&mut self, data: &[u8]) {
        self.on_done.signal(PathBuf::from(str::from_utf8(data).unwrap()));
    }

    fn do_cancel(&mut self) {
        self.on_cancel.signal(None);
    }
}

signal_decl!{ConfigSetEvent(String)}

pub struct ConfigSetLine {
    pub on_done: ConfigSetEvent,
    pub on_cancel: Canceled,
    prefix: String,
}

impl ConfigSetLine {
    pub fn new(prefix: String) -> ConfigSetLine {
        ConfigSetLine {
            prefix: prefix,
            on_done: Default::default(),
            on_cancel: Default::default(),
        }
    }
}

impl InputLineBehavior for ConfigSetLine {
    fn get_prefix(&mut self) -> String {
        self.prefix.clone()
    }

    fn do_enter(&mut self, data: &[u8]) {
        self.on_done.signal(str::from_utf8(&data).unwrap().to_owned());
    }

    fn do_cancel(&mut self) {
        self.on_cancel.signal(None);
    }
}
