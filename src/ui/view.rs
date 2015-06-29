use std::cmp;
use std::path::Path;
use std::path::PathBuf;
use util::string_with_repeat;
use std::error::Error;
use rustbox::{RustBox, Color, RB_NORMAL, RB_BOLD};

use super::super::buffer::Buffer;
use super::super::segment::Segment;
use super::super::signals;

use super::common::{Rect, u8_to_hex};
use super::inputline::{InputLine, GotoInputLine, FindInputLine, PathInputLine};
use super::overlay::OverlayText;

#[derive(Debug)]
enum UndoAction {
    Delete(isize, isize),
    Insert(isize, Vec<u8>),
    Write(isize, Vec<u8>)
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

    pub fn draw(&mut self, rb: &RustBox) {
        let nibble_view_start = self.nibble_start;
        let byte_view_start = nibble_view_start + (self.nibble_width / 2) * 3;

        let mut prev_in_selection = false;

        let extra_none: &[Option<&u8>] = &[None];

        let start_iter = (self.data_offset / 2) as usize;
        let stop_iter = cmp::min(start_iter + (self.nibble_size / 2) as usize, self.buffer.len());

        for (byte_i, maybe_byte) in self.buffer.iter_range(start_iter, stop_iter)
        // This is needed for the "fake" last element for insertion mode
            .map(|x| Some(x))
            .chain(extra_none.iter().map(|n| *n))
            .enumerate() {

            let row = byte_i as isize / (self.nibble_width / 2);
            let offset = byte_i as isize % (self.nibble_width / 2);
            let byte_pos = byte_i as isize + self.data_offset / 2;

            if offset == 0 {
                if self.nibble_start == 5 {
                    rb.print(0, row as usize, RB_NORMAL, Color::White, Color::Black,
                             &format!("{:04X}", byte_pos));
                } else {
                    rb.print(0, row as usize, RB_NORMAL, Color::White, Color::Black,
                             &format!("{:04X}:{:04X}", byte_pos >> 16, byte_pos & 0xFFFF));
                }
            }

            let mut s = String::new();
            let mut byte_str = ".".to_string();
            match maybe_byte {
                Some(&byte) => {
                    let (char_0, char_1) = u8_to_hex(byte);
                    s.push(char_0);
                    s.push(char_1);
                    let alphanumeric =
                        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                        .find(byte as char).is_some();
                    if alphanumeric {
                        byte_str = String::from_utf8(vec!(byte)).unwrap()
                    }
                }

                // Then this is the last iteration so that insertion past the last byte works
                None => {
                    s.push(' ');
                    byte_str = " ".to_string();
                }
            }

            let at_current_byte = byte_pos == (self.cursor_pos / 2);

            let mut in_selection = false;
            match self.selection_start {
                Some(selection_pos) if selection_pos / 2 < self.cursor_pos / 2
                    => in_selection =
                           (selection_pos / 2 <= byte_pos) && (byte_pos <= self.cursor_pos / 2),
                       Some(selection_pos) => in_selection = (self.cursor_pos / 2 <= byte_pos) &&
                                                             (byte_pos <= selection_pos / 2),
                                              None => ()
            }

            let nibble_view_pos = [
                (nibble_view_start + (offset * 3)) as usize, row as usize
            ];
            let nibble_colors = if (!self.nibble_active && at_current_byte) || in_selection {
                [Color::Black, Color::White]
            } else {
                [Color::White, Color::Black]
            };

            rb.print(nibble_view_pos[0], nibble_view_pos[1] as usize, RB_NORMAL, nibble_colors[0],
                     nibble_colors[1], &s);
            if prev_in_selection && in_selection {
                rb.print(nibble_view_pos[0] - 1, nibble_view_pos[1] as usize, RB_NORMAL,
                         nibble_colors[0], nibble_colors[1], " ");

            }
            if self.nibble_active && self.input_entry.is_none() && at_current_byte {
                rb.set_cursor(nibble_view_pos[0] as isize + (self.cursor_pos & 1),
                              nibble_view_pos[1] as isize);
            };

            prev_in_selection = in_selection;

            let byte_colors = if (self.nibble_active && at_current_byte) || in_selection {
                [Color::Black, Color::White]
            } else {
                [Color::White, Color::Black]
            };

            rb.print((byte_view_start + offset) as usize, row as usize, RB_NORMAL, byte_colors[0],
                     byte_colors[1], &byte_str);
            if !self.nibble_active && self.input_entry.is_none() && at_current_byte {
                rb.set_cursor(byte_view_start + offset, row);
            }
        }

        match self.input_entry.as_mut() {
            Some(entry) => entry.draw(rb, Rect {
                top: (rb.height() - 2) as isize,
                bottom: (rb.height() - 1) as isize,
                left: 0,
                right: rb.width() as isize
            },
                                      true),
            None => ()
        };

        match self.overlay.as_mut() {
            Some(entry) => entry.draw(rb, Rect {
                top: 0,
                bottom: self.cur_height,
                left: 0,
                right: self.cur_width,
            },
                                      true),
            None => ()
        };

        rb.print(0, rb.height() - 1, RB_NORMAL, Color::Black, Color::White,
                 &string_with_repeat(' ', rb.width()));
        match self.status_log.last() {
            Some(ref status_line) => rb.print(0, rb.height() - 1, RB_NORMAL, Color::Black,
                                              Color::White, &status_line),
            None => (),
        }
        let right_status = format!(
            "overlay = {:?}, input = {:?} undo = {:?}, pos = {:?}, selection = {:?}, insert = {:?}",
            self.overlay.is_none(), self.input_entry.is_none(), self.undo_stack.len(),
            self.cursor_pos, self.selection_start, self.insert_mode);
        // let lll = self.buffer.segment._internal_debug();
        // let right_status = format!("clip = {}, vecs = {}", self.clipboard.is_some(), lll);
        rb.print(rb.width() - right_status.len(), rb.height() - 1, RB_NORMAL, Color::Black,
                 Color::White, &right_status);
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

    fn view_input(&mut self, emod: u8, key: u16, ch: u32) {
        let printable = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                        .find((ch as u8) as char).is_some();
        match (emod, key, ch) {
            // Movement
            (0, 0xFFEB, _) if self.nibble_active => self.move_cursor(-1),
            (0, 0xFFEA, _) if self.nibble_active => self.move_cursor(1),
            (0, 0xFFEB, _) if !self.nibble_active => self.move_cursor(-2),
            (0, 0xFFEA, _) if !self.nibble_active => self.move_cursor(2),
            (0, 0xFFED, _) => {
                let t = -self.nibble_width;
                self.move_cursor(t)
            }
            (0, 0xFFEC, _) => {
                let t = self.nibble_width;
                self.move_cursor(t)
            }

            (0, 0xFFEF, _) => {
                let t = -(self.nibble_size - self.nibble_width) / 2;
                self.move_cursor(t)
            }
            (0, 0xFFEE, _) => {
                let t = (self.nibble_size - self.nibble_width) / 2;
                self.move_cursor(t)
            }

            // UndoAction::Delete
            (0, 0xFFF2, _) => self.delete_at_cursor(false),
            (0, 127, 0) => self.delete_at_cursor(true),

            // Ctrl X, C V
            (0, 24, 0) => self.edit_cut(),
            (0, 3, 0) => self.edit_copy(),
            (0, 22, 0) => self.edit_paste(),

            // Hex input for nibble view
            (0, 0, 48...57) if self.nibble_active => {
                self.write_nibble_at_cursor((ch - 48) as u8);
                self.move_cursor(1)
            }
            (0, 0, 97...102) if self.nibble_active => {
                self.write_nibble_at_cursor((ch - 97 + 10) as u8);
                self.move_cursor(1)
            }
            (0, 0, 65...70) if self.nibble_active => {
                self.write_nibble_at_cursor((ch - 65 + 10) as u8);
                self.move_cursor(1)
            }
            (0, 0, _) if !self.nibble_active && printable => {
                self.write_byte_at_cursor(ch as u8);
                self.move_cursor(2);
            },

            (0, 9, 0) => {
                self.nibble_active = !self.nibble_active;
                let t = self.nibble_active;
                self.status(format!("nibble_active = {:?}", t));
            },

            (0, 31, 0) => self.start_help(),

            (0, 15, 0) => self.toggle_insert_mode(),

            (0, 19, 0) => self.toggle_selection(),

            (0, 26, 0) => self.undo(),

            (0, 7, 0) => self.start_goto(),
            (0, 6, 0) => self.start_find(),
            (0, 5, 0) => self.start_open(),
            (0, 23, 0) => self.start_save(),

            _ => self.status(format!("emod = {:?}, key = {:?}, ch = {:?}", emod, key, ch)),
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

    pub fn input(&mut self, emod: u8, key: u16, ch: u32) {
        self.process_msgs();

        match self.overlay {
            Some(ref mut overlay) => {
                overlay.input(emod, key, ch);
                return;
            }
            None => ()
        }

        match self.input_entry {
            Some(ref mut input_entry) => {
                input_entry.input(emod, key, ch);
                return;
            }
            None => ()
        }

        self.view_input(emod, key, ch);

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
