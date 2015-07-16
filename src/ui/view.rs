use std::str;
use std::cmp;
use std::path::Path;
use std::path::PathBuf;
use util::{string_with_repeat, is_between};
use std::error::Error;
use std::ascii::AsciiExt;
use rustbox::{RustBox};
use rustbox::keyboard::Key;


use super::super::buffer::Buffer;
use super::super::segment::Segment;

use super::common::{Rect, u8_to_hex};
use super::RustBoxEx::{RustBoxEx, Style};
use super::input::Input;
use super::inputline::{InputLine, GotoInputLine, FindInputLine, PathInputLine};
use super::overlay::OverlayText;

#[derive(Debug)]
enum UndoAction {
    Delete(isize, isize),
    Insert(isize, Vec<u8>),
    Write(isize, Vec<u8>),
}

#[derive(Copy,Clone,Debug)]
pub enum HexEditActions {
    Edit(char),
    SwitchView,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MovePageUp,
    MovePageDown,
    Delete,
    DeleteWithMove,
    CopySelection,
    CutSelection,
    PasteSelection,
    Undo,
    ToggleInsert,
    ToggleSelecion,
    HelpView,
    AskGoto,
    AskFind,
    AskOpen,
    AskSave
}

signalreceiver_decl!{HexEditSignalReceiver(HexEdit)}

pub struct HexEdit {
    buffer: Segment,
    cursor_pos: isize,
    cur_height: isize,
    cur_width: isize,
    nibble_width: isize,
    nibble_size: isize,
    data_size: isize,
    status_log: Vec<String>,
    data_offset: isize,
    nibble_start: isize,
    nibble_active: bool,
    selection_start: Option<isize>,
    insert_mode: bool,
    input: Input,
    undo_stack: Vec<UndoAction>,
    input_entry: Option<Box<InputLine>>,
    overlay: Option<OverlayText>,
    cur_path: Option<PathBuf>,
    clipboard: Option<Vec<u8>>,

    signal_receiver: Option<HexEditSignalReceiver>,
}

impl HexEdit {
    pub fn new() -> HexEdit {
        HexEdit {
            buffer: Segment::new(),
            cursor_pos: 0,
            nibble_size: 0,
            cur_width: 50,
            cur_height: 50,
            nibble_width: 1,
            data_offset: 0,
            nibble_start: 0,
            data_size: 0,
            status_log: vec!("Press C-/ for help".to_string()),
            nibble_active: true,
            selection_start: None,
            insert_mode: false,
            input_entry: None,
            undo_stack: Vec::new(),
            overlay: None,
            cur_path: None,
            clipboard: None,
            input: Input::new(),
            signal_receiver: Some(HexEditSignalReceiver::new()),
        }
    }

    fn reset(&mut self) {
        self.cursor_pos = 0;
        self.data_offset = 0;
        self.nibble_active = true;
        self.selection_start = None;
        self.insert_mode = false;
        self.input_entry = None;
        self.undo_stack = Vec::new();
        self.recalculate();
    }

    pub fn draw_view(&mut self, rb: &RustBox) {
        let nibble_view_start = self.nibble_start as usize;
        let byte_view_start = nibble_view_start + (self.nibble_width as usize / 2) * 3;

        let mut prev_in_selection = false;

        let extra_none: &[Option<&u8>] = &[None];

        let start_iter = (self.data_offset / 2) as usize;
        let stop_iter = cmp::min(start_iter + (self.nibble_size / 2) as usize, self.buffer.len());

        for (byte_i, maybe_byte) in self.buffer.iter_range(start_iter, stop_iter)
        // This is needed for the "fake" last element for insertion mode
            .map(|x| Some(x))
            .chain(extra_none.iter().map(|n| *n))
            .enumerate() {

            let row = byte_i / (self.nibble_width as usize / 2);
            let column = byte_i % (self.nibble_width as usize / 2);
            let byte_pos = byte_i as isize + self.data_offset / 2;

            // It's a new row, let's draw the line numbers
            if column == 0 {
                if self.nibble_start == 5 {
                    rb.print_style(0, row, Style::Default, &format!("{:04X}", byte_pos));
                } else {
                    rb.print_style(0, row, Style::Default, &format!("{:04X}:{:04X}", byte_pos >> 16, byte_pos & 0xFFFF));
                }
                // We want the selection draw to not go out of the editor view
                prev_in_selection = false;
            }

            let at_current_byte = byte_pos == (self.cursor_pos / 2);

            let in_selection = if let Some(selection_pos) = self.selection_start {
                is_between(byte_pos, selection_pos / 2, self.cursor_pos / 2)
            } else {
                false
            };

            // Now we draw the nibble view
            let hex_chars = if let Some(&byte) = maybe_byte {
                u8_to_hex(byte)
            } else {
                (' ', ' ')
            };

            let nibble_view_column = nibble_view_start + (column * 3);
            let nibble_style = if (!self.nibble_active && at_current_byte) || in_selection {
                Style::Selection
            } else {
                Style::Default
            };

            rb.print_char_style(nibble_view_column, row, nibble_style,
                hex_chars.0);
            rb.print_char_style(nibble_view_column + 1, row, nibble_style,
                hex_chars.1);
            if prev_in_selection && in_selection {
                rb.print_char_style(nibble_view_column - 1, row, nibble_style,
                    ' ');

            }
            if self.nibble_active && self.input_entry.is_none() && at_current_byte {
                rb.set_cursor(nibble_view_column as isize + (self.cursor_pos & 1),
                              row as isize);
            };

            // Now let's draw the byte window

            let byte_char = if let Some(&byte) = maybe_byte {
                let bc = byte as char;
                if bc.is_ascii() && bc.is_alphanumeric() {
                    bc
                } else {
                    ' '
                }
            } else {
                ' '
            };

            // If we are at the current byte but the nibble view is active, we want to draw a
            // "fake" cursor by dawing a selection square
            let byte_style = if (self.nibble_active && at_current_byte) || in_selection {
                Style::Selection
            } else {
                Style::Default
            };

            rb.print_char_style(byte_view_start + column, row, byte_style,
                byte_char);
            if !self.nibble_active && self.input_entry.is_none() && at_current_byte {
                rb.set_cursor((byte_view_start + column) as isize, row as isize);
            }

            // Remember if we had a selection, so that we know for next char to "fill in" with
            // selection in the nibble view
            prev_in_selection = in_selection;
        }
    }

    fn draw_statusbar(&mut self, rb: &RustBox) {
        rb.print_style(0, rb.height() - 1, Style::StatusBar, &string_with_repeat(' ', rb.width()));
        match self.status_log.last() {
            Some(ref status_line) => rb.print_style(0, rb.height() - 1, Style::StatusBar, &status_line),
            None => (),
        }
        let right_status = format!(
            "overlay = {:?}, input = {:?} undo = {:?}, pos = {:?}, selection = {:?}, insert = {:?}",
            self.overlay.is_none(), self.input_entry.is_none(), self.undo_stack.len(),
            self.cursor_pos, self.selection_start, self.insert_mode);
        rb.print_style(rb.width() - right_status.len(), rb.height() - 1, Style::StatusBar, &right_status);        
    }

    pub fn draw(&mut self, rb: &RustBox) {
        self.draw_view(rb);

        if let Some(entry) = self.input_entry.as_mut() {
            entry.draw(rb, Rect {
                top: (rb.height() - 2) as isize,
                bottom: (rb.height() - 1) as isize,
                left: 0,
                right: rb.width() as isize
            }, true);
        }

        if let Some(overlay) = self.overlay.as_mut() {
            overlay.draw(rb, Rect {
                top: 0,
                bottom: self.cur_height,
                left: 0,
                right: self.cur_width,
            }, true);
        }

        self.draw_statusbar(rb);
    }

    fn status(&mut self, st: String) {
        self.status_log.push(st);
    }

    pub fn open(&mut self, path: &Path) {
        match Segment::from_path(path) {
            Ok(buf) => {
                self.buffer = buf;
                self.cur_path = Some(PathBuf::from(path));
                self.reset();
            }
            Err(e) => {
                self.status(format!("ERROR: {}", e.description()));
            }
        }
    }

    pub fn save(&mut self, path: &Path) {
        match self.buffer.save(path) {
            Ok(_) => {
                self.cur_path = Some(PathBuf::from(path));
            }
            Err(e) => {
                self.status(format!("ERROR: {}", e.description()));
            }
        }
    }

    fn do_action(&mut self, act: UndoAction, add_to_undo: bool) -> (isize, isize) {
        let stat = format!("doing = {:?}", act);
        let mut begin_region: isize;
        let mut end_region: isize;

        match act {
            UndoAction::Insert(offset, buf) => {
                begin_region = offset;
                end_region = offset + buf.len() as isize;

                self.buffer.insert(offset as usize, &buf);
                if add_to_undo {
                    self.push_undo(UndoAction::Delete(offset, offset + buf.len() as isize))
                }
                self.recalculate();
            }
            UndoAction::Delete(offset, end) => {
                begin_region = offset;
                end_region = end;

                let res = self.buffer.remove(offset as usize, end as usize);
                if add_to_undo { self.push_undo(UndoAction::Insert(offset, res)) }
                self.recalculate();
            }
            UndoAction::Write(offset, buf) => {
                begin_region = offset;
                end_region = offset + buf.len() as isize;

                let orig_data = self.buffer.read(offset as usize, buf.len());
                self.buffer.write(offset as usize, &buf);
                if add_to_undo { self.push_undo(UndoAction::Write(offset, orig_data)) }
            }
        }

        self.status(stat);
        (begin_region, end_region)
    }

    fn push_undo(&mut self, act: UndoAction) {
        self.undo_stack.push(act);
    }

    fn undo(&mut self) {
        match self.undo_stack.pop() {
            Some(act) => {
                let (begin, _) = self.do_action(act, false);
                self.set_cursor(begin * 2);
            }
            None => ()
        }
    }

    fn cursor_at_end(&self) -> bool {
        self.cursor_pos == self.data_size
    }

    fn delete_at_cursor(&mut self, with_bksp: bool) {
        let mut cursor_pos = self.cursor_pos;

        let selection_pos = match self.selection_start {
            Some(selection_pos_tag) => selection_pos_tag,
            None => {
                if with_bksp {
                    if cursor_pos < 2 {
                        return;
                    }
                    cursor_pos -= 2;
                }
                cursor_pos
            }
        };

        let del_start = cmp::min(selection_pos, cursor_pos) / 2;
        let mut del_stop = cmp::max(selection_pos, cursor_pos) / 2 + 1;

        if del_stop > self.data_size / 2 {
            del_stop -= 1;
            if del_stop == del_start {
                return;
            }
        }

        if self.data_size == 0 {
            self.status(format!("Nothing to delete"));
            return;
        }

        self.selection_start = None;
        self.do_action(UndoAction::Delete(del_start, del_stop), true);
        self.set_cursor(del_start * 2);
    }

    fn write_nibble_at_cursor(&mut self, c: u8) {
        match self.selection_start {
            Some(_) => self.delete_at_cursor(false),
            None => ()
        }

        if self.insert_mode || self.cursor_at_end() {
            self.insert_nibble_at_cursor(c);
        } else {
            self.set_nibble_at_cursor(c);
        }
    }

    fn set_nibble_at_cursor(&mut self, c: u8) {
        let mut byte = self.buffer[(self.cursor_pos / 2) as usize];

        byte = match self.cursor_pos & 1 {
            0 => (byte & 0x0f) + c * 16,
            1 => (byte & 0xf0) + c,
            _ => 0xff,
        };

        let byte_offset = self.cursor_pos / 2;
        self.do_action(UndoAction::Write(byte_offset, vec!(byte)), true);
    }

    fn insert_nibble_at_cursor(&mut self, c: u8) {
        // If we are at half byte, we still overwrite
        if self.cursor_pos & 1 == 1 {
            self.set_nibble_at_cursor(c);
            return
        }

        let pos_div2 = self.cursor_pos / 2;
        self.do_action(UndoAction::Insert(pos_div2, vec!(c * 16)), true);
    }

    fn toggle_insert_mode(&mut self) {
        self.insert_mode = !self.insert_mode;
        self.move_cursor(0);
    }

    fn write_byte_at_cursor(&mut self, c: u8) {
        match self.selection_start {
            Some(_) => self.delete_at_cursor(false),
            None => ()
        }

        let byte_offset = self.cursor_pos / 2;
        if self.insert_mode || self.cursor_at_end() {
            self.do_action(UndoAction::Insert(byte_offset, vec!(c)), true);
        } else {
            self.do_action(UndoAction::Write(byte_offset, vec!(c)), true);
        }
    }

    fn move_cursor(&mut self, pos: isize) {
        self.cursor_pos += pos;
        self.update_cursor()
    }

    fn set_cursor(&mut self, pos: isize) {
        self.cursor_pos = pos;
        self.update_cursor()
    }

    fn update_cursor(&mut self) {

        self.cursor_pos = cmp::max(self.cursor_pos, 0);
        self.cursor_pos = cmp::min(self.cursor_pos, self.data_size);

        if self.cursor_pos < self.data_offset {
            self.data_offset = (self.cursor_pos / self.nibble_width) * self.nibble_width;
        }

        if self.cursor_pos > (self.data_offset + self.nibble_size - 1) {
            let end_row = self.cursor_pos - (self.cursor_pos % self.nibble_width) -
                          self.nibble_size + self.nibble_width;
            self.data_offset = end_row;
        }
    }

    fn toggle_selection(&mut self) {
        match self.selection_start {
            Some(_) => self.selection_start = None,
            None => self.selection_start = Some(self.cursor_pos)
        }
        let st = format!("selection = {:?}", self.selection_start);
        self.status(st.clone());
    }

    fn goto(&mut self, pos: isize) {
        self.status(format!("Going to {:?}", pos));
        self.set_cursor(pos * 2);
    }

    fn find_buf(&mut self, needle: &[u8]) {
        let found_pos = match self.buffer.find_from((self.cursor_pos / 2) as usize, needle) {
            None => {
                self.buffer.find_from(0, needle)
            }
            a => a
        };

        match found_pos {
            Some(pos) => {
                self.status(format!("Found at {:?}", pos));
                self.set_cursor((pos * 2) as isize);
            }
            None => {
                self.status(format!("Nothing found!"));
            }
        };
    }

    fn read_cursor_to_clipboard(&mut self) -> Option<usize> {
        let (start, stop) = match self.selection_start {
            None => { return None; },
            Some(selection_pos) => {
                (cmp::min(selection_pos, self.cursor_pos) / 2,
                 cmp::max(selection_pos, self.cursor_pos) / 2)
            }
        };

        let data = self.buffer.read(start as usize, stop as usize);
        let data_len = data.len();

        self.clipboard = Some(data);
        Some(data_len)
    }

    fn edit_copy(&mut self) {
        match self.read_cursor_to_clipboard() {
            Some(data_len) => self.status(format!("Copied {}", data_len)),
            None => ()
        }
    }

    fn edit_cut(&mut self) {
        match self.read_cursor_to_clipboard() {
            Some(data_len) => {
                self.delete_at_cursor(false);
                self.status(format!("Cut {}", data_len));
            }
            None => ()
        }
    }

    fn edit_paste(&mut self) {
        let data;
        match self.clipboard {
            Some(ref d) => { data = d.clone(); },
            None => { return; }
        };

        let pos_div2 = self.cursor_pos / 2;
        self.do_action(UndoAction::Insert(pos_div2, data), true);
    }

    fn view_input(&mut self, key: Key) {
        let action = self.input.editor_input(key);
        if action.is_none() {
            return;
        }
        match action.unwrap() {
            // Movement
            HexEditActions::MoveLeft if self.nibble_active => self.move_cursor(-1),
            HexEditActions::MoveRight if self.nibble_active => self.move_cursor(1),
            HexEditActions::MoveLeft if !self.nibble_active => self.move_cursor(-2),
            HexEditActions::MoveRight if !self.nibble_active => self.move_cursor(2),
            HexEditActions::MoveUp => {
                let t = -self.nibble_width;
                self.move_cursor(t)
            }
            HexEditActions::MoveDown => {
                let t = self.nibble_width;
                self.move_cursor(t)
            }

            HexEditActions::MovePageUp => {
                let t = -(self.nibble_size - self.nibble_width) / 2;
                self.move_cursor(t)
            }
            HexEditActions::MovePageDown => {
                let t = (self.nibble_size - self.nibble_width) / 2;
                self.move_cursor(t)
            }

            // UndoAction::Delete
            HexEditActions::Delete => self.delete_at_cursor(false),
            HexEditActions::DeleteWithMove => self.delete_at_cursor(true),

            // Ctrl X, C V
            HexEditActions::CutSelection => self.edit_cut(),
            HexEditActions::CopySelection => self.edit_copy(),
            HexEditActions::PasteSelection => self.edit_paste(),

            // Hex input for nibble view
            HexEditActions::Edit(ch) if self.nibble_active => {
                match ch.to_digit(16) {
                    Some(val) => {
                        self.write_nibble_at_cursor(val as u8);
                        self.move_cursor(1);
                    }
                    None => ()  // TODO: Show error?
                }
            },

            // Ascii edit for byte view
            HexEditActions::Edit(ch) if !self.nibble_active => {
                if ch.len_utf8() == 1 && ch.is_alphanumeric() {
                    // TODO: Make it printable rather than alphanumeric
                    self.write_byte_at_cursor(ch as u8);
                    self.move_cursor(2);
                } else {
                    // TODO: Show error?
                }
            }

            HexEditActions::SwitchView => {
                self.nibble_active = !self.nibble_active;
                let t = self.nibble_active;
                self.status(format!("nibble_active = {:?}", t));
            },

            HexEditActions::HelpView => self.start_help(),

            HexEditActions::ToggleInsert => self.toggle_insert_mode(),

            HexEditActions::ToggleSelecion => self.toggle_selection(),

            HexEditActions::Undo => self.undo(),

            HexEditActions::AskGoto => self.start_goto(),
            HexEditActions::AskFind => self.start_find(),
            HexEditActions::AskOpen => self.start_open(),
            HexEditActions::AskSave => self.start_save(),

            _  => self.status(format!("key = {:?}", key)),
        }
    }

    fn start_help(&mut self) {
        let help_text = include_str!("Help.txt");
        let ref sr = self.signal_receiver.as_mut().unwrap();
        let mut ot = OverlayText::with_text(help_text.to_string());
        ot.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            match opt_msg {
                Some(ref msg) => obj.status(msg.clone()),
                None => ()
            };
            obj.overlay = None;
        }));
        self.overlay = Some(ot);
    }

    fn start_goto(&mut self) {
        let mut gt = GotoInputLine::new();
        // let mut sender_clone0 = self.sender.clone();
        let ref sr = self.signal_receiver.as_mut().unwrap();
        gt.on_done.connect(signal!(sr with |obj, pos| {
            obj.goto(pos*2);
            obj.input_entry = None;
        }));

        gt.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            match opt_msg {
                Some(ref msg) => obj.status(msg.clone()),
                None => ()
            };
            obj.input_entry = None;
        }));

        self.input_entry = Some(Box::new(gt) as Box<InputLine>)
    }

    fn start_find(&mut self) {
        let mut find_line = FindInputLine::new();
        let ref sr = self.signal_receiver.as_mut().unwrap();
        find_line.on_find.connect(signal!(sr with |obj, needle| {
            obj.find_buf(&needle);
            obj.input_entry = None;
        }));

        find_line.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            match opt_msg {
                Some(ref msg) => obj.status(msg.clone()),
                None => ()
            };
            obj.input_entry = None;
        }));

        self.input_entry = Some(Box::new(find_line) as Box<InputLine>)
    }

    fn start_save(&mut self) {
        let mut path_line = PathInputLine::new("Save: ".into());
        let ref sr = self.signal_receiver.as_mut().unwrap();
        path_line.on_done.connect(signal!(sr with |obj, path| {
            obj.save(&path);
            obj.input_entry = None;
        }));

        path_line.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            match opt_msg {
                Some(ref msg) => obj.status(msg.clone()),
                None => ()
            };
            obj.input_entry = None;
        }));

        self.input_entry = Some(Box::new(path_line) as Box<InputLine>)
    }

    fn start_open(&mut self) {
        let mut path_line = PathInputLine::new("Open: ".into());
        let ref sr = self.signal_receiver.as_mut().unwrap();
        path_line.on_done.connect(signal!(sr with |obj, path| {
            obj.open(&path);
            obj.input_entry = None;
        }));

        path_line.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            match opt_msg {
                Some(ref msg) => obj.status(msg.clone()),
                None => ()
            };
            obj.input_entry = None;
        }));

        self.input_entry = Some(Box::new(path_line) as Box<InputLine>)
    }

    fn process_msgs(&mut self) {
        let mut sr = self.signal_receiver.take().unwrap();
        sr.run(self);
        self.signal_receiver = Some(sr);
    }

    pub fn input(&mut self, key: Key) {
        self.process_msgs();

        match self.overlay {
            Some(ref mut overlay) => {
                overlay.input(&self.input, key);
                return;
            }
            None => ()
        }

        match self.input_entry {
            Some(ref mut input_entry) => {
                input_entry.input(&self.input, key);
                return;
            }
            None => ()
        }

        self.view_input(key);

        self.process_msgs();
    }

    fn recalculate(&mut self) {
        self.data_size = (self.buffer.len() * 2) as isize;
        let (new_width, new_height) = (self.cur_width as i32, (self.cur_height + 1) as i32);
        self.resize(new_width, new_height);
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.cur_height = (height as isize) - 1;
        self.cur_width = width as isize;
        self.nibble_start = if self.data_size / 2 <= 0xFFFF { 1 + 4 } else { 2 + 8 };
        self.nibble_width = 2 * ((self.cur_width - self.nibble_start) / 4);
        self.nibble_size = self.nibble_width * self.cur_height;
    }
}
