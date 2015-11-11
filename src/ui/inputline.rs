use std::str;
use std::io;
use std::fs;
use rustc_serialize::hex::FromHex;
use std::path::{PathBuf, Path};
use std::marker::PhantomData;
use std::error::Error;

use toml;

use rex_utils;
use rex_utils::rect::Rect;
use super::super::frontend::{Frontend, Style, KeyPress};
use super::super::filesystem::Filesystem;
use super::super::config::Value;
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
    fn get_prefix(&self) -> &str;
    fn get_status(&self) -> Result<&str, &str> {
        Ok("")
    }
    fn do_update(&mut self, data: &[u8]) {

    }
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
                    self.behavior.do_update(&self.data);
                }
            }
        };

        self.behavior.do_update(&self.data);

        return true;
    }

    fn draw(&mut self, rb: &mut Frontend, area: Rect<isize>, has_focus: bool) {
        let prefix = self.behavior.get_prefix();

        let (style, status_msg) = match self.behavior.get_status() {
            Ok(sm) => (Style::InputLine, sm),
            Err(sm) => (Style::InputLineError, sm),
        };

        let (x_pos, start_index) = if area.width >= status_msg.len() as isize {
            (area.width - status_msg.len() as isize, 0)
        } else {
            (0, area.width - status_msg.len() as isize)
        };

        rb.print_style(area.left as usize, area.top as usize, style,
                 &rex_utils::string_with_repeat(' ', area.width as usize));
        rb.print_style(x_pos as usize, area.top as usize, style,
              &status_msg[start_index as usize..]);
        rb.print_style(area.left as usize, area.top as usize, style,
                 &format!("{}{} ", prefix, str::from_utf8(&self.data).unwrap()));
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
    is_valid: bool,
    pub on_done: GotoEvent,
    pub on_cancel: Canceled,
}

impl GotoInputLineBehavior {
    pub fn new() -> GotoInputLineBehavior {
        GotoInputLineBehavior {
            radix: RadixType::DecRadix,
            is_valid: true,
            on_done: Default::default(),
            on_cancel: Default::default(),
        }
    }

    fn set_radix(&mut self, r: RadixType) {
        self.radix = r;
    }

    fn get_pos(&mut self, data: &[u8]) -> Option<isize> {
        let radix = match self.radix {
            RadixType::DecRadix => 10,
            RadixType::HexRadix => 16,
            RadixType::OctRadix => 8,
        };

        match str::from_utf8(&data) {
            Ok(gs) => isize::from_str_radix(&gs, radix).ok(),
            Err(_) => None
        }
    }

    fn do_goto(&mut self, data: &[u8]) {
        match self.get_pos(data) {
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
    fn get_prefix(&self) -> &str {
        match self.radix {
            RadixType::DecRadix => "Goto (Dec):",
            RadixType::HexRadix => "Goto (Hex):",
            RadixType::OctRadix => "Goto (Oct):",
        }
    }

    fn get_status(&self) -> Result<&str, &str> {
        if self.is_valid {
            Ok("")
        } else {
            Err("Invalid position")
        }
    }

    fn do_update(&mut self, data: &[u8]) {
        self.is_valid = self.get_pos(data).is_some();
    }

    fn do_enter(&mut self, data: &[u8]) {
        if self.is_valid {
            self.do_goto(data);
        }
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DataType {
    AsciiStr,
    UnicodeStr,
    HexStr,
}

signal_decl!{FindEvent(Vec<u8>)}

pub struct FindInputLine {
    data_type: DataType,
    is_valid: bool,
    pub on_find: FindEvent,
    pub on_cancel: Canceled,
}

impl FindInputLine {
    pub fn new() -> FindInputLine {
        FindInputLine {
            data_type: DataType::AsciiStr,
            is_valid: true,
            on_find: Default::default(),
            on_cancel: Default::default(),
        }
    }

    fn set_search_data_type(&mut self, dt: DataType) {
        self.data_type = dt;
    }

    fn parse_hex(&self, data: &[u8]) -> Option<Vec<u8>> {
        str::from_utf8(data).unwrap().from_hex().ok()
    }

    fn do_find(&mut self, data: &[u8]) {
        let ll = self.parse_hex(data);

        let needle: Vec<u8> = match self.data_type {
            DataType::AsciiStr => data.clone().into(),
            DataType::UnicodeStr => data.clone().into(),
            DataType::HexStr => {
                match ll {
                    Some(n) => n,
                    None => {
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
    fn get_prefix(&self) -> &str {
        match self.data_type {
            DataType::AsciiStr => "Find(Ascii): ",
            DataType::UnicodeStr => "Find(Uni): ",
            DataType::HexStr => "Find(Hex): ",
        }
    }

    fn get_status(&self) -> Result<&str, &str> {
        if self.is_valid {
            Ok("")
        } else {
            Err("Invalid Hex Value")
        }
    }


    fn do_update(&mut self, data: &[u8]) {
        self.is_valid = (self.data_type != DataType::HexStr) || self.parse_hex(data).is_some();
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PathInputType {
    Open,
    Save
}

signal_decl!{PathEvent(PathBuf)}

pub struct PathInputLine<FS: Filesystem> {
    pub on_done: PathEvent,
    pub on_cancel: Canceled,
    input_type: PathInputType,
    res: Option<String>,

    _fs: PhantomData<FS>
}

impl<FS: Filesystem> PathInputLine<FS> {
    pub fn new(input_type: PathInputType) -> PathInputLine<FS> {
        PathInputLine {
            input_type: input_type,
            on_done: Default::default(),
            on_cancel: Default::default(),
            res: None,

            _fs: PhantomData,
        }
    }
}

impl<FS: Filesystem> InputLineBehavior for PathInputLine<FS> {
    fn get_prefix(&self) -> &str {
        if self.input_type == PathInputType::Open {
            "Open: "
        } else {
            "Save: "
        }
    }

    fn get_status(&self) -> Result<&str, &str> {
        if let Some(ref s) = self.res {
            Err(s)
        } else {
            Ok("")
        }
    }

    fn do_update(&mut self, data: &[u8]) {
        self.res = if self.input_type == PathInputType::Open {
            FS::can_open(Path::new(str::from_utf8(data).unwrap())).err().map(|e| format!("{}", e))
        } else {
            FS::can_save(Path::new(str::from_utf8(data).unwrap())).err().map(|e| format!("{}", e))
        }
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
    pub fn new(prefix: String, value: Value) -> ConfigSetLine {
        ConfigSetLine {
            prefix: prefix,
            on_done: Default::default(),
            on_cancel: Default::default(),
        }
    }
}

impl InputLineBehavior for ConfigSetLine {
    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn do_enter(&mut self, data: &[u8]) {
        self.on_done.signal(str::from_utf8(&data).unwrap().to_owned());
    }

    fn do_cancel(&mut self) {
        self.on_cancel.signal(None);
    }
}
