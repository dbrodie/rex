use std::cmp;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::iter;
use std::error::Error;
use std::ascii::AsciiExt;
use itertools::Itertools;
use std::borrow::Cow;
use rustbox::{RustBox};
use rustbox::keyboard::Key;

use rex_utils;
use rex_utils::split_vec::SplitVec;
use rex_utils::rect::Rect;
use super::super::config::Config;

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

#[derive(Debug)]
enum LineNumberMode {
    None,
    Short,
    Long
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
    MoveToFirstColumn,
    MoveToLastColumn,
    Delete,
    DeleteWithMove,
    CopySelection,
    CutSelection,
    PasteSelection,
    Undo,
    ToggleInsert,
    ToggleSelecion,
    HelpView,
    LogView,
    AskGoto,
    AskFind,
    AskOpen,
    AskSave
}

signalreceiver_decl!{HexEditSignalReceiver(HexEdit)}

pub struct HexEdit {
    buffer: SplitVec,
    config: Config,
    rect: Rect<isize>,
    cursor_nibble_pos: isize,
    status_log: Vec<String>,
    show_last_status: bool,
    data_offset: isize,
    row_offset: isize,
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
    pub fn new(config: Config) -> HexEdit {
        HexEdit {
            buffer: SplitVec::new(),
            config: config,
            rect: Default::default(),
            cursor_nibble_pos: 0,
            data_offset: 0,
            row_offset: 0,
            status_log: vec!["Press C-/ for help".to_string()],
            show_last_status: true,
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
        self.cursor_nibble_pos = 0;
        self.data_offset = 0;
        self.nibble_active = true;
        self.selection_start = None;
        self.insert_mode = false;
        self.input_entry = None;
        self.undo_stack = Vec::new();
    }

    fn get_linenumber_mode(&self) -> LineNumberMode {
        if !self.config.show_linenum {
            LineNumberMode::None
        } else if self.buffer.len() <= 0xFFFF {
            LineNumberMode::Short
        } else {
            LineNumberMode::Long
        }
    }

    fn get_linenumber_width(&self) -> isize {
        match self.get_linenumber_mode() {
            LineNumberMode::None => 1,
            LineNumberMode::Short => 4 + 1, // 4 for the XXXX + 1 for whitespace
            LineNumberMode::Long => 9 + 1, // 7 for XXXX:XXXX + 1 for whitespace
        }
    }

    fn get_line_width(&self) -> isize {
        self.config.line_width.unwrap_or(self.get_bytes_per_row() as u32) as isize
    }

    fn get_bytes_per_row(&self) -> isize {
        // This is the number of cells on the screen that are used for each byte.
        // For the nibble view, we need 3 (1 for each nibble and 1 for the spacing). For
        // the ascii view, if it is shown, we need another one.
        let cells_per_byte = if self.config.show_ascii { 4 } else { 3 };

        (self.rect.width - self.get_linenumber_width()) / cells_per_byte
    }

    fn get_bytes_per_screen(&self) -> isize {
        self.get_line_width() * self.rect.height
    }

    fn draw_line_number(&self, rb: &RustBox, row: usize, line_number: usize) {
        match self.get_linenumber_mode() {
            LineNumberMode::None => (),
            LineNumberMode::Short => {
                rb.print_style(0, row, Style::Default, &format!("{:04X}", line_number));
            }
            LineNumberMode::Long => {
                rb.print_style(0, row, Style::Default, &format!("{:04X}:{:04X}", line_number >> 16, line_number & 0xFFFF));
            }
        };
    }

    fn draw_line(&self, rb: &RustBox, iter: &mut Iterator<Item=(usize, Option<&u8>)>, row: usize) {
        let nibble_view_start = self.get_linenumber_width() as usize;
        // The value of this is wrong if we are not showing the ascii view
        let byte_view_start = nibble_view_start + self.get_bytes_per_row() as usize * 3;

        // We want the selection draw to not go out of the editor view
        let mut prev_in_selection = false;
        let mut at_current_row = false;

        for (row_offset, (byte_pos, maybe_byte)) in iter.skip(self.row_offset as usize).enumerate().take(self.get_bytes_per_row() as usize) {
            let at_current_byte = byte_pos as isize == (self.cursor_nibble_pos / 2);
            at_current_row = at_current_row || at_current_byte;

            let in_selection = if let Some(selection_pos) = self.selection_start {
                rex_utils::is_between(byte_pos as isize, selection_pos, self.cursor_nibble_pos / 2)
            } else {
                false
            };

            // Now we draw the nibble view
            let hex_chars = if let Some(&byte) = maybe_byte {
                rex_utils::u8_to_hex(byte)
            } else {
                (' ', ' ')
            };

            let nibble_view_column = nibble_view_start + (row_offset * 3);
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
                rb.set_cursor(nibble_view_column as isize + (self.cursor_nibble_pos & 1),
                              row as isize);
            };

            if self.config.show_ascii {
                // Now let's draw the byte window
                let byte_char = if let Some(&byte) = maybe_byte {
                    let bc = byte as char;
                    if bc.is_ascii() && bc.is_alphanumeric() {
                        bc
                    } else {
                        '.'
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

                rb.print_char_style(byte_view_start + row_offset, row, byte_style,
                    byte_char);
                if !self.nibble_active && self.input_entry.is_none() && at_current_byte {
                    rb.set_cursor((byte_view_start + row_offset) as isize, row as isize);
                }

                // Remember if we had a selection, so that we know for next char to "fill in" with
                // selection in the nibble view
                prev_in_selection = in_selection;
            }
        }

        // We just need to consume the iterator and see if there were any remaining bytes
        let bytes_remaining = iter.count();

        if at_current_row && self.row_offset != 0 {
            rb.print_char_style(nibble_view_start - 1, row, Style::Default, '<');
        }
        if at_current_row && bytes_remaining != 0 {
            rb.print_char_style(byte_view_start - 1, row, Style::Default, '>');
        }
    }

    pub fn draw_view(&self, rb: &RustBox) {
        let start_iter = self.data_offset as usize;
        let stop_iter = cmp::min(start_iter + self.get_bytes_per_screen() as usize, self.buffer.len());

        let itit = (start_iter..).zip(  // We are zipping the byte position
            self.buffer.iter_range(start_iter..stop_iter)  // With the data at those bytes
            .map(|x| Some(x))  // And wrapping it in an option
            .chain(iter::once(None)))  // So we can have a "fake" last item that will be None
            .chunks_lazy(self.get_line_width() as usize);  //And split it into nice row-sized chunks

        for (row, row_iter_) in itit.into_iter().take(self.rect.height as usize).enumerate() {
            // We need to be able to peek in the iterable so we can get the current position
            let mut row_iter = row_iter_.peekable();
            let byte_pos = row_iter.peek().unwrap().0;
            self.draw_line_number(rb, row, byte_pos);

            self.draw_line(rb, &mut row_iter, row);
        }
    }

    fn draw_statusbar(&self, rb: &RustBox) {
        rb.print_style(0, rb.height() - 1, Style::StatusBar, &rex_utils::string_with_repeat(' ', rb.width()));
        if self.show_last_status {
            if let Some(ref status_line) = self.status_log.last() {
                rb.print_style(0, rb.height() - 1, Style::StatusBar, &status_line);
            }
        }

        let mode = if let Some(_) = self.selection_start {
            "SEL"
        } else if self.insert_mode {
            "INS"
        } else {
            "OVR"
        };

        let right_status;
        if let Some(selection_start) = self.selection_start {
            let size = (self.cursor_nibble_pos/2 - selection_start).abs();
            right_status = format!(
                " Start: {} Size: {} Pos: {} {}",
                selection_start, size, self.cursor_nibble_pos/2, mode);
        } else {
            right_status = format!(
                " Pos: {} Undo: {} {}",
                self.undo_stack.len(), self.cursor_nibble_pos/2, mode);
        };
        let (x_pos, start_index) = if rb.width() >= right_status.len() {
            (rb.width() - right_status.len(), 0)
        } else {
            (0, right_status.len() - rb.width())
        };
        rb.print_style(x_pos, rb.height() - 1, Style::StatusBar, &right_status[start_index..]);
    }

    pub fn draw(&mut self, rb: &RustBox) {
        self.draw_view(rb);

        if let Some(entry) = self.input_entry.as_mut() {
            entry.draw(rb, Rect {
                top: (rb.height() - 2) as isize,
                left: 0,
                height: 1,
                width: rb.width() as isize
            }, true);
        }

        if let Some(overlay) = self.overlay.as_mut() {
            overlay.draw(rb, Rect {
                top: 0,
                left: 0,
                height: self.rect.height,
                width: self.rect.width,
            }, true);
        }

        self.draw_statusbar(rb);
    }

    fn status<S: Into<Cow<'static, str>> + ?Sized>(&mut self, st: S) {
            self.show_last_status = true;
            let cow: Cow<'static, str> = st.into();
            self.status_log.push(format!("{}", &cow));
        }

    fn clear_status(&mut self) {
        self.show_last_status = false;
    }

    pub fn open(&mut self, path: &Path) {
        let mut v = vec![];
        if let Err(e) = File::open(path).and_then(|mut f| f.read_to_end(&mut v)) {
            self.status(format!("ERROR: {}", e.description()));
            return;
        }
        self.buffer = SplitVec::from_vec(v);
        self.cur_path = Some(PathBuf::from(path));
        self.reset();
    }

    pub fn save(&mut self, path: &Path) {
        let result = File::create(path)
            .and_then(|mut f| self.buffer.iter_slices()
                      .fold(Ok(()), |res, val| res
                            .and_then(|_| f.write_all(val))));

        match result {
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
            }
            UndoAction::Delete(offset, end) => {
                begin_region = offset;
                end_region = end;

                let res = self.buffer.move_out(offset as usize..end as usize);
                if add_to_undo { self.push_undo(UndoAction::Insert(offset, res)) }
            }
            UndoAction::Write(offset, buf) => {
                begin_region = offset;
                end_region = offset + buf.len() as isize;

                let orig_data = self.buffer.copy_out(offset as usize..(offset as usize + buf.len()));
                self.buffer.copy_in(offset as usize, &buf);
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
        if let Some(act) = self.undo_stack.pop() {
            let (begin, _) = self.do_action(act, false);
            self.set_cursor(begin * 2);
        }
    }

    fn cursor_at_end(&self) -> bool {
        self.cursor_nibble_pos == (self.buffer.len()*2) as isize
    }

    fn delete_at_cursor(&mut self, with_bksp: bool) {
        let mut cursor_nibble_pos = self.cursor_nibble_pos;

        let selection_pos = match self.selection_start {
            Some(selection_pos_tag) => selection_pos_tag,
            None => {
                if with_bksp {
                    if cursor_nibble_pos < 2 {
                        return;
                    }
                    cursor_nibble_pos -= 2;
                }
                cursor_nibble_pos / 2
            }
        };

        let del_start = cmp::min(selection_pos, cursor_nibble_pos / 2);
        let mut del_stop = cmp::max(selection_pos, cursor_nibble_pos / 2) + 1;

        if del_stop > self.buffer.len() as isize {
            del_stop -= 1;
            if del_stop == del_start {
                return;
            }
        }

        if self.buffer.len() == 0 {
            self.status("Nothing to delete");
            return;
        }

        self.selection_start = None;
        self.do_action(UndoAction::Delete(del_start, del_stop), true);
        self.set_cursor(del_start * 2);
    }

    fn write_nibble_at_cursor(&mut self, c: u8) {
        // Replace the text at the selection before writing the data
        if self.selection_start.is_some() {
            self.delete_at_cursor(false);
        }

        if self.insert_mode || self.cursor_at_end() {
            self.insert_nibble_at_cursor(c);
        } else {
            self.set_nibble_at_cursor(c);
        }
    }

    fn set_nibble_at_cursor(&mut self, c: u8) {
        let mut byte = self.buffer[(self.cursor_nibble_pos / 2) as usize];

        byte = match self.cursor_nibble_pos & 1 {
            0 => (byte & 0x0f) + c * 16,
            1 => (byte & 0xf0) + c,
            _ => 0xff,
        };

        let byte_offset = self.cursor_nibble_pos / 2;
        self.do_action(UndoAction::Write(byte_offset, vec![byte]), true);
    }

    fn insert_nibble_at_cursor(&mut self, c: u8) {
        // If we are at half byte, we still overwrite
        if self.cursor_nibble_pos & 1 == 1 {
            self.set_nibble_at_cursor(c);
            return
        }

        let pos_div2 = self.cursor_nibble_pos / 2;
        self.do_action(UndoAction::Insert(pos_div2, vec![c * 16]), true);
    }

    fn toggle_insert_mode(&mut self) {
        self.insert_mode = !self.insert_mode;
        self.move_cursor(0);
    }

    fn write_byte_at_cursor(&mut self, c: u8) {
        // Replace the text at the selection before writing the data
        if self.selection_start.is_some() {
            self.delete_at_cursor(false);
        }

        let byte_offset = self.cursor_nibble_pos / 2;
        if self.insert_mode || self.cursor_at_end() {
            self.do_action(UndoAction::Insert(byte_offset, vec![c]), true);
        } else {
            self.do_action(UndoAction::Write(byte_offset, vec![c]), true);
        }
    }

    fn move_cursor(&mut self, pos: isize) {
        self.cursor_nibble_pos += pos;
        self.update_cursor()
    }

    fn set_cursor(&mut self, pos: isize) {
        self.cursor_nibble_pos = pos;
        self.update_cursor()
    }

    fn update_cursor(&mut self) {
        self.cursor_nibble_pos = cmp::max(self.cursor_nibble_pos, 0);
        self.cursor_nibble_pos = cmp::min(self.cursor_nibble_pos, (self.buffer.len()*2) as isize);
        let cursor_byte_pos = self.cursor_nibble_pos / 2;
        let cursor_row_offset = cursor_byte_pos % self.get_line_width();

        // If the cursor moves above or below the view, scroll it
        if cursor_byte_pos < self.data_offset {
            self.data_offset = (cursor_byte_pos) - cursor_row_offset;
        }

        if cursor_byte_pos > (self.data_offset + self.get_bytes_per_screen() - 1) {
            self.data_offset = cursor_byte_pos  - cursor_row_offset -
                          self.get_bytes_per_screen() + self.get_line_width();
        }

        // If the cursor moves to the right or left of the view, scroll it
        if cursor_row_offset < self.row_offset {
            self.row_offset = cursor_row_offset;
        }
        if cursor_row_offset >= self.row_offset + self.get_bytes_per_row() {
            self.row_offset = cursor_row_offset - self.get_bytes_per_row() + 1;
        }
    }

    fn toggle_selection(&mut self) {
        match self.selection_start {
            Some(_) => self.selection_start = None,
            None => self.selection_start = Some(self.cursor_nibble_pos / 2)
        }
        let selection_start = self.selection_start; // Yay! Lifetimes!
        self.status(format!("selection = {:?}", selection_start));
    }

    fn goto(&mut self, pos: isize) {
        self.status(format!("Going to {:?}", pos));
        self.set_cursor(pos * 2);
    }

    fn find_buf(&mut self, needle: &[u8]) {
        let found_pos = match self.buffer.find_slice_from((self.cursor_nibble_pos / 2) as usize, needle) {
            None => {
                self.buffer.find_slice_from(0, needle)
            }
            a => a
        };

        if let Some(pos) = found_pos {
            self.status(format!("Found at {:?}", pos));
            self.set_cursor((pos * 2) as isize);
        } else {
            self.status("Nothing found!");
        }
    }

    fn read_cursor_to_clipboard(&mut self) -> Option<usize> {
        let (start, stop) = match self.selection_start {
            None => { return None; },
            Some(selection_pos) => {
                (cmp::min(selection_pos, self.cursor_nibble_pos / 2),
                 cmp::max(selection_pos, self.cursor_nibble_pos / 2))
            }
        };

        let data = self.buffer.copy_out(start as usize..stop as usize);
        let data_len = data.len();

        self.clipboard = Some(data);
        Some(data_len)
    }

    fn edit_copy(&mut self) {
        if let Some(data_len) = self.read_cursor_to_clipboard() {
             self.status(format!("Copied {}", data_len));
             self.selection_start = None;
        }
    }

    fn edit_cut(&mut self) {
        if let Some(data_len) = self.read_cursor_to_clipboard() {
            self.delete_at_cursor(false);
            self.status(format!("Cut {}", data_len));
        }
    }

    fn edit_paste(&mut self) {
        let data = if let Some(ref d) = self.clipboard {
            d.clone()
        } else {
            return;
        };

        let data_len = data.len() as isize;
        // This is needed to satisfy the borrow checker
        let cur_pos_in_bytes = self.cursor_nibble_pos / 2;
        self.do_action(UndoAction::Insert(cur_pos_in_bytes, data), true);
        self.move_cursor(data_len + 1);
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
                let t = -self.get_line_width() * 2;
                self.move_cursor(t)
            }
            HexEditActions::MoveDown => {
                let t = self.get_line_width() * 2;
                self.move_cursor(t)
            }

            HexEditActions::MovePageUp => {
                let t = -(self.get_bytes_per_screen() * 2);
                self.move_cursor(t)
            }
            HexEditActions::MovePageDown => {
                let t = self.get_bytes_per_screen() * 2;
                self.move_cursor(t)
            }
            HexEditActions::MoveToFirstColumn => {
                let pos_in_line = self.cursor_nibble_pos % (self.get_line_width()*2);
                self.move_cursor(-pos_in_line)
            }
            HexEditActions::MoveToLastColumn => {
                let pos_in_line = self.cursor_nibble_pos % (self.get_line_width()*2);
                let i = self.get_line_width()*2 - 2 - pos_in_line;
                self.move_cursor(i);
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
                if let Some(val) = ch.to_digit(16) {
                    self.write_nibble_at_cursor(val as u8);
                    self.move_cursor(1);
                } else {
                    // TODO: Show error?
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
            HexEditActions::LogView => self.start_logview(),

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
        // YAY Lifetimes! (This will hopfully be fixed once rust gains MIR/HIR)
        {
            let ref sr = self.signal_receiver.as_mut().unwrap();
            let mut ot = OverlayText::with_text(help_text.to_string(), false);
            ot.on_cancel.connect(signal!(sr with |obj, opt_msg| {
                if let Some(ref msg) = opt_msg {
                    obj.status(msg.clone());
                } else {
                    obj.clear_status();
                }
                obj.overlay = None;
            }));
            self.overlay = Some(ot);
        }
        {
            self.status("Press Esc to return");
        }
    }

    fn start_logview(&mut self) {
        let logs = self.status_log.clone();
        let ref sr = self.signal_receiver.as_mut().unwrap();
        let mut ot = OverlayText::with_logs(logs, true);
        ot.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
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
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
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
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
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
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
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
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
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

        if let Some(ref mut overlay) = self.overlay {
            overlay.input(&self.input, key);
        } else if let Some(ref mut input_entry) = self.input_entry {
            input_entry.input(&self.input, key);
        } else {
            self.view_input(key);
        }

        self.process_msgs();
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.rect.height = height as isize - 1;  // Substract 1 for the status line on the bottom
        self.rect.width = width as isize;
        self.update_cursor();
    }
}
