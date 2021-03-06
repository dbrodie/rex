use std::cmp;
use std::io;
use std::io::Read;
use std::io::Write;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;
use std::iter;
use std::ascii::AsciiExt;
use itertools::Itertools;
use std::borrow::Cow;
use std::rc::Rc;
use std::marker::PhantomData;

use util;
use util::split_vec::SplitVec;
use util::rect::Rect;
use util::relative_rect::{RelativeRect, RelativePos, RelativeSize};
use util::signals::SignalReceiver;
use super::super::config::{Config, Value, ConfigError};

use super::super::frontend::{Frontend, Style, KeyPress};
use super::super::filesystem::{Filesystem, DefaultFilesystem};
use super::input::Input;
use super::widget::Widget;
use super::inputline::{
    InputLine,
    GotoInputLineBehavior,
    FindInputLine,
    PathInputLine,
    PathInputType,
    ConfigSetLine,
};
use super::overlay::OverlayText;
use super::configscreen::ConfigScreen;
use super::menu::{OverlayMenu, MenuState, MenuEntry};


custom_derive! {
    /// An offset into the buffer in nibbles.
    /// All offsets into the buffer are kept as bytes. In the few cases where it is easier to keep
    /// it as an offset in nibbles, we will use this type to differentiate.
    #[derive(NewtypeAdd, NewtypeSub, Clone, Copy, Debug, PartialEq, Eq)]
    struct Nibble(isize);
}

impl Nibble {
    fn from_bytes(byte_pos: isize) -> Nibble {
        Nibble(byte_pos * 2)
    }

    fn to_bytes(&self) -> isize {
        self.0 / 2
    }

    /// Return the low bit of the nibble offset, meaning which nibble the offset is on in the byte
    /// offset.
    fn nibble_bit(&self) -> u8 {
        (self.0 & 1) as u8
    }
}

/// Represents an edit operation done in a buffer, such as paste, insertion and deletion.
/// Undo operations are also saved as EditOperations that revert the original operation.
#[derive(Debug, Clone)]
struct EditOperation {
    /// The range over which the operation happens
    range: Range<usize>,
    /// The data that replaced the supplied range
    data: Vec<u8>,
    /// A short description of the type of operation for logging
    description: &'static str,
}

impl EditOperation {
    fn delete(range: Range<usize>) -> EditOperation {
        EditOperation {
            range: range,
            data : vec![],
            description: "Delete",
        }
    }

    fn insert(offset: usize, data: Vec<u8>) -> EditOperation {
        EditOperation {
            range: offset..offset,
            data: data,
            description: "Insert",
        }
    }

    fn write(offset: usize, data: Vec<u8>) -> EditOperation {
        EditOperation {
            range: offset..(offset + data.len()),
            data: data,
            description: "Overwrite",
        }
    }
}

#[derive(Debug)]
enum LineNumberMode {
    None,
    Short,
    Long
}

static OVERLAY_LAYOUT : RelativeRect<isize> = RelativeRect {
    top: RelativePos::FromStart(0),
    left: RelativePos::FromStart(0),
    width: RelativeSize::Relative(0),
    height: RelativeSize::Relative(0),
};

static INPUTLINE_LAYOUT : RelativeRect<isize> = RelativeRect {
    top: RelativePos::FromEnd(1),
    left: RelativePos::FromStart(0),
    width: RelativeSize::Relative(0),
    height: RelativeSize::Absolute(1),
};

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
    AskSave,
    AskConfig,
    AskMarkAdd,
    AskMarkGoto,
    StartMenu,
}

static ROOT_ENTRIES: MenuState<HexEditActions> = &[
    MenuEntry::CommandEntry('c', "Config", HexEditActions::AskConfig),
    MenuEntry::SubEntries('m', "Mark", &[
        MenuEntry::CommandEntry('a', "Add", HexEditActions::AskMarkAdd),
        MenuEntry::CommandEntry('g', "Goto", HexEditActions::AskMarkGoto),
    ]),
];

pub struct HexEdit<FS: Filesystem+'static = DefaultFilesystem> {
    buffer: SplitVec,
    config: Rc<Config<FS>>,
    rect: Rect<isize>,
    /// The cursor position in nibbles
    cursor_nibble_pos: Nibble,
    status_log: Vec<String>,
    show_last_status: bool,
    data_offset: isize,
    row_offset: isize,
    nibble_active: bool,
    selection_start: Option<isize>,
    insert_mode: bool,
    input: Input,
    undo_stack: Vec<EditOperation>,
    child_widget: Option<(Box<Widget>, RelativeRect<isize>)>,
    cur_path: Option<PathBuf>,
    clipboard: Option<Vec<u8>>,

    signal_receiver: Rc<SignalReceiver<HexEdit<FS>>>,
    _fs: PhantomData<FS>,
}

impl<FS: Filesystem+'static> HexEdit<FS> {
    pub fn new() -> HexEdit<FS> {
        let (config, err_msg) = match Config::open_default() {
            Ok(config) => (config, None),
            Err(ConfigError::IoError(ref err)) if err.kind() == io::ErrorKind::NotFound =>
                (Default::default(), None),
            Err(err) => (Default::default(), Some(err)),
        };
        let mut h = HexEdit {
            buffer: SplitVec::new(),
            config: Rc::new(config),
            rect: Default::default(),
            cursor_nibble_pos: Nibble(0),
            data_offset: 0,
            row_offset: 0,
            status_log: vec!["Press C-/ for help".to_string()],
            show_last_status: true,
            nibble_active: true,
            selection_start: None,
            insert_mode: false,
            child_widget: None,
            undo_stack: Vec::new(),
            cur_path: None,
            clipboard: None,
            input: Input::new(),
            signal_receiver: Rc::new(SignalReceiver::new()),
            _fs: PhantomData,
        };
        if let Some(err) = err_msg {
            h.status(format!("Error opening config: {}", err));
        }
        h
    }

    fn reset(&mut self) {
        self.cursor_nibble_pos = Nibble(0);
        self.data_offset = 0;
        self.nibble_active = true;
        self.selection_start = None;
        self.insert_mode = false;
        self.child_widget = None;
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
        let byte_width = self.rect.width - self.get_linenumber_width();
        // The number of cells per byte WITHOUT whitespace is dependent on wether we are showing
        // the ascii bytes or not.
        let cells_per_byte = if self.config.show_ascii { 3 } else { 2 };
        // The number of cells to display each group is dependent on the cells_per_byte and the
        // bytes per group with an added whitespace char between the groups in hex view.
        let cells_per_group = self.config.group_bytes as isize * (cells_per_byte) + 1;
        let num_of_groups = byte_width / cells_per_group;
        let cells_per_byte =  num_of_groups * self.config.group_bytes as isize;
        cells_per_byte
    }

    fn get_bytes_per_screen(&self) -> isize {
        self.get_line_width() * self.rect.height
    }

    fn draw_line_number(&self, rb: &mut Frontend, row: usize, line_number: usize) {
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

    /// Helper function that returns the cell offset a nibble view should be displayed at
    fn nibble_view_column(&self, row_offset: usize) -> usize {
        // Two cells for each byte and the byte offset divided the number of bytes per group will
        // give us the number of whitespace characters used.
        self.get_linenumber_width() as usize + row_offset * 2 + (row_offset / self.config.group_bytes as usize)
    }

    fn draw_line(&self, rb: &mut Frontend, iter: &mut Iterator<Item=(usize, Option<&u8>)>, row: usize) {
        let nibble_view_start = self.get_linenumber_width() as usize;
        // The value of this is wrong if we are not showing the ascii view
        let byte_view_start = self.nibble_view_column(self.get_bytes_per_row() as usize);

        // We want the selection draw to not go out of the editor view
        let mut prev_in_selection = false;
        let mut at_current_row = false;

        for (row_offset, (byte_pos, maybe_byte)) in iter.skip(self.row_offset as usize).enumerate().take(self.get_bytes_per_row() as usize) {
            let at_current_byte = byte_pos as isize == self.cursor_nibble_pos.to_bytes();
            at_current_row = at_current_row || at_current_byte;

            let in_selection = if let Some(selection_pos) = self.selection_start {
                util::is_between(byte_pos as isize, selection_pos, self.cursor_nibble_pos.to_bytes())
            } else {
                false
            };

            // Now we draw the nibble view
            let hex_chars = if let Some(&byte) = maybe_byte {
                util::u8_to_hex(byte)
            } else {
                (' ', ' ')
            };

            let nibble_view_column;
            if !self.config.little_endian {
                nibble_view_column = self.nibble_view_column(row_offset);
            } else {
                // Reverse the order of bytes in case of little endian
                let group_offset = row_offset % self.config.group_bytes as usize;
                let opposite_group_offset = self.config.group_bytes as usize - group_offset - 1;
                nibble_view_column = self.nibble_view_column(row_offset - group_offset + opposite_group_offset);
            }
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
            if self.nibble_active && self.child_widget.is_none() && at_current_byte {
                rb.set_cursor(nibble_view_column as isize + self.cursor_nibble_pos.nibble_bit() as isize,
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
                if !self.nibble_active && self.child_widget.is_none() && at_current_byte {
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

    pub fn draw_view(&self, rb: &mut Frontend) {
        let start_iter = self.data_offset as usize;
        let stop_iter = cmp::min(start_iter + self.get_bytes_per_screen() as usize, self.buffer.len());

        let itit = (start_iter..).zip(  // We are zipping the byte position
            self.buffer.iter_range(start_iter..stop_iter)  // With the data at those bytes
            .map(Some)  // And wrapping it in an option
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

    fn draw_statusbar(&self, rb: &Frontend) {
        rb.print_style(0, rb.height() - 1, Style::StatusBar, &util::string_with_repeat(' ', rb.width()));
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
            let size = (self.cursor_nibble_pos.to_bytes() - selection_start).abs();
            right_status = format!(
                " Start: {} Size: {} Pos: {} {}",
                selection_start, size, self.cursor_nibble_pos.to_bytes(), mode);
        } else {
            right_status = format!(
                " Pos: {} Undo: {} {}",
                self.cursor_nibble_pos.to_bytes(), self.undo_stack.len(), mode);
        };
        let (x_pos, start_index) = if rb.width() >= right_status.len() {
            (rb.width() - right_status.len(), 0)
        } else {
            (0, right_status.len() - rb.width())
        };
        rb.print_style(x_pos, rb.height() - 1, Style::StatusBar, &right_status[start_index..]);
    }

    pub fn draw(&mut self, rb: &mut Frontend) {
        self.draw_view(rb);

        if let Some(&mut (ref mut child_widget, ref layout)) = self.child_widget.as_mut() {
            child_widget.draw(rb, layout.get_absolute_to(self.rect), true);
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

    pub fn open_vec(&mut self, vec: Vec<u8>) {
        self.buffer = SplitVec::from_vec(vec);
        self.reset();
    }

    pub fn open(&mut self, path: &Path) {
        let mut v = vec![];
        if let Err(e) = FS::open(path).and_then(|mut f| f.read_to_end(&mut v)) {
            self.status(format!("ERROR: {}", e));
            return;
        }
        self.buffer = SplitVec::from_vec(v);
        self.cur_path = Some(PathBuf::from(path));
        self.reset();
    }

    pub fn save(&mut self, path: &Path) {
        let result = FS::save(path)
            .and_then(|mut f| self.buffer.iter_slices()
                      .fold(Ok(()), |res, val| res
                            .and_then(|_| f.write_all(val))
                        )
                    );

        match result {
            Ok(_) => {
                self.cur_path = Some(PathBuf::from(path));
            }
            Err(e) => {
                self.status(format!("ERROR: {}", e));
            }
        }
    }

    /// We pretty much apply the data over the range as a splice, except for when an operation
    /// exceeds the end of the vector, and then we will cap the range to the length of the vector
    fn edit_buffer(&mut self, operation: EditOperation, add_to_undo: bool) {
        let begin = operation.range.start;
        let orig_data = self.buffer.splice(operation.range, &operation.data);
        if add_to_undo {
            let undo_operation = EditOperation {
                range: begin..operation.data.len(),
                data: orig_data,
                description: operation.description,
            };
            self.push_undo(undo_operation);
        }
    }

    fn push_undo(&mut self, operation: EditOperation) {
        self.undo_stack.push(operation);
    }

    fn undo(&mut self) {
        if let Some(operation) = self.undo_stack.pop() {
            let begin = operation.range.start;
            self.edit_buffer(operation, false);
            self.set_cursor(Nibble::from_bytes(begin as isize));
        }
    }

    fn cursor_at_end(&self) -> bool {
        self.cursor_nibble_pos == Nibble::from_bytes(self.buffer.len() as isize)
    }

    fn delete_at_cursor(&mut self, with_bksp: bool) {
        let mut cursor_nibble_pos = self.cursor_nibble_pos;

        let selection_pos = match self.selection_start {
            Some(selection_pos_tag) => selection_pos_tag,
            None => {
                if with_bksp {
                    // If we are at the begining of the document, there isn't what to backspace into
                    if cursor_nibble_pos.to_bytes() < 1 {
                        return;
                    }
                    cursor_nibble_pos = cursor_nibble_pos - Nibble::from_bytes(1);
                }
                cursor_nibble_pos.to_bytes()
            }
        };

        let del_start = cmp::min(selection_pos, cursor_nibble_pos.to_bytes());
        let mut del_stop = cmp::max(selection_pos, cursor_nibble_pos.to_bytes()) + 1;

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
        self.edit_buffer(EditOperation::delete(del_start as usize..del_stop as usize), true);
        self.set_cursor(Nibble::from_bytes(del_start));
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
        let mut byte = self.buffer[self.cursor_nibble_pos.to_bytes() as usize];

        byte = match self.cursor_nibble_pos.nibble_bit() {
            0 => (byte & 0x0f) + c * 16,
            1 => (byte & 0xf0) + c,
            _ => 0xff,
        };

        let byte_offset = self.cursor_nibble_pos.to_bytes();
        self.edit_buffer(EditOperation::write(byte_offset as usize, vec![byte]), true);
    }

    fn insert_nibble_at_cursor(&mut self, c: u8) {
        // If we are at half byte, we still overwrite
        if self.cursor_nibble_pos.nibble_bit() == 1 {
            self.set_nibble_at_cursor(c);
            return
        }

        let pos_div2 = self.cursor_nibble_pos.to_bytes();
        self.edit_buffer(EditOperation::insert(pos_div2 as usize, vec![c * 16]), true);
    }

    fn toggle_insert_mode(&mut self) {
        self.insert_mode = !self.insert_mode;
    }

    fn write_byte_at_cursor(&mut self, c: u8) {
        // Replace the text at the selection before writing the data
        if self.selection_start.is_some() {
            self.delete_at_cursor(false);
        }

        let byte_offset = self.cursor_nibble_pos.to_bytes();
        if self.insert_mode || self.cursor_at_end() {
            self.edit_buffer(EditOperation::insert(byte_offset as usize, vec![c]), true);
        } else {
            self.edit_buffer(EditOperation::write(byte_offset as usize, vec![c]), true);
        }
    }

    fn move_cursor(&mut self, pos: Nibble) {
        self.cursor_nibble_pos = self.cursor_nibble_pos + pos;
        self.update_cursor()
    }

    fn set_cursor(&mut self, pos: Nibble) {
        self.cursor_nibble_pos = pos;
        self.update_cursor()
    }

    fn update_cursor(&mut self) {
        self.cursor_nibble_pos = Nibble(cmp::max(self.cursor_nibble_pos.0, 0));
        self.cursor_nibble_pos = Nibble(cmp::min(self.cursor_nibble_pos.0, Nibble::from_bytes(self.buffer.len()as isize).0));
        let cursor_byte_pos = self.cursor_nibble_pos.to_bytes();
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
            None => self.selection_start = Some(self.cursor_nibble_pos.to_bytes())
        }
    }

    fn goto(&mut self, pos: isize) {
        self.status(format!("Going to {:?}", pos));
        self.set_cursor(Nibble::from_bytes(pos));
    }

    fn find_buf(&mut self, needle: &[u8]) {
        let found_pos = match self.buffer.find_slice_from(self.cursor_nibble_pos.to_bytes() as usize, needle) {
            None => {
                self.buffer.find_slice_from(0, needle)
            }
            a => a
        };

        if let Some(pos) = found_pos {
            self.status(format!("Found at {:?}", pos));
            self.set_cursor(Nibble::from_bytes(pos as isize));
        } else {
            self.status("Nothing found!");
        }
    }

    fn read_cursor_to_clipboard(&mut self) -> Option<usize> {
        let (start, stop) = match self.selection_start {
            None => { return None; },
            Some(selection_pos) => {
                (cmp::min(selection_pos, self.cursor_nibble_pos.to_bytes()),
                 cmp::max(selection_pos, self.cursor_nibble_pos.to_bytes()))
            }
        };

        let data = self.buffer.copy_out(start as usize..(stop + 1) as usize);
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
        let cur_pos_in_bytes = self.cursor_nibble_pos.to_bytes();
        if self.insert_mode {
            self.edit_buffer(EditOperation::insert(cur_pos_in_bytes as usize, data), true);
        } else {
            self.edit_buffer(EditOperation::write(cur_pos_in_bytes as usize, data), true);
        }
        self.move_cursor(Nibble::from_bytes(data_len));
    }

    fn view_input(&mut self, key: KeyPress) {
        if let Some(action) = self.input.editor_input(key) {
            self.do_action(action)
        }
    }

    fn do_action(&mut self, action: HexEditActions) {
        self.clear_status();
        match action {
            // Movement
            HexEditActions::MoveLeft if self.nibble_active => self.move_cursor(Nibble(-1)),
            HexEditActions::MoveRight if self.nibble_active => self.move_cursor(Nibble(1)),
            HexEditActions::MoveLeft => self.move_cursor(Nibble::from_bytes(-1)),
            HexEditActions::MoveRight => self.move_cursor(Nibble::from_bytes(1)),

            HexEditActions::MoveUp => {
                let t = -self.get_line_width();
                self.move_cursor(Nibble::from_bytes(t))
            }
            HexEditActions::MoveDown => {
                let t = self.get_line_width();
                self.move_cursor(Nibble::from_bytes(t))
            }

            HexEditActions::MovePageUp => {
                let t = -self.get_bytes_per_screen();
                self.move_cursor(Nibble::from_bytes(t))
            }
            HexEditActions::MovePageDown => {
                let t = self.get_bytes_per_screen();
                self.move_cursor(Nibble::from_bytes(t))
            }
            HexEditActions::MoveToFirstColumn => {
                let pos_in_line = self.cursor_nibble_pos.to_bytes() % self.get_line_width();
                self.move_cursor(Nibble::from_bytes(-pos_in_line))
            }
            HexEditActions::MoveToLastColumn => {
                let pos_in_line = self.cursor_nibble_pos.to_bytes() % self.get_line_width();
                let i = self.get_line_width() - 1 - pos_in_line;
                self.move_cursor(Nibble::from_bytes(i));
            }

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
                    self.move_cursor(Nibble(1));
                } else {
                    // TODO: Show error?
                }
            },

            // Ascii edit for byte view
            HexEditActions::Edit(ch) => {
                if ch.len_utf8() == 1 && ch.is_alphanumeric() {
                    // TODO: Make it printable rather than alphanumeric
                    self.write_byte_at_cursor(ch as u8);
                    self.move_cursor(Nibble::from_bytes(1));
                } else {
                    // TODO: Show error?
                }
            }

            HexEditActions::SwitchView => {
                self.nibble_active = !self.nibble_active;
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
            HexEditActions::AskConfig => self.start_config(),

            HexEditActions::StartMenu => self.start_menu(),

            _ => self.status(format!("Operation not implemented yet: {:?}", action))
        }
    }

    fn start_menu(&mut self) {
        let sr = &self.signal_receiver;
        let mut menu = OverlayMenu::with_menu(ROOT_ENTRIES);
        menu.on_selected.connect(signal!(sr with |obj, action| {
            obj.child_widget = None;
            obj.do_action(action);
        }));
        menu.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));
        self.child_widget = Some((Box::new(menu), OVERLAY_LAYOUT));
    }

    fn start_config(&mut self) {
        let sr = &self.signal_receiver;
        let mut config_screen = ConfigScreen::with_config(self.config.clone());
        config_screen.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));
        config_screen.on_selected.connect(signal!(sr with |obj, conf_name, conf_val| {
            obj.child_widget = None;
            obj.start_config_edit(conf_name, conf_val.clone());
        }));
        self.child_widget = Some((Box::new(config_screen), OVERLAY_LAYOUT));
    }

    fn start_config_edit(&mut self, conf_name: &'static str, conf_value: Value) {
        let sr = &self.signal_receiver;
        let initial_val = format!("{}", conf_value).into_bytes();
        let mut config_set = ConfigSetLine::new(format!("{} = ", conf_name), conf_value);
        config_set.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));
        config_set.on_done.connect(signal!(sr with |obj, config_value| {
            obj.child_widget = None;
            obj.set_config(conf_name, &config_value);
        }));
        self.child_widget = Some((Box::new(InputLine::new_with_value(config_set, initial_val)), INPUTLINE_LAYOUT));
    }

    /// Setting the config is only "allowed" from the main view, and all child widgets should have
    /// been removed meanwhile.
    fn set_config(&mut self, key: &str, val: &str) {
        let res = Rc::get_mut(& mut self.config).unwrap().set_from_key_value(key, &val);
        res.unwrap_or_else(
            |e| self.status(format!("Can't set {} to {}: {}", key, val, e))
        );
        self.config.save_default().unwrap_or_else(
            |e| self.status(format!("Can't save config: {}", e))
        );
    }

    fn start_help(&mut self) {
        let help_text = include_str!("Help.txt");
        // YAY Lifetimes! (This will hopfully be fixed once rust gains MIR/HIR)
        {
            let sr = &self.signal_receiver;
            let mut ot = OverlayText::with_text(help_text.to_string(), false);
            ot.on_cancel.connect(signal!(sr with |obj, opt_msg| {
                obj.child_widget = None;
                if let Some(ref msg) = opt_msg {
                    obj.status(msg.clone());
                } else {
                    obj.clear_status();
                }
            }));
            self.child_widget = Some((Box::new(ot), OVERLAY_LAYOUT));
        }
        {
            self.status("Press Esc to return");
        }
    }

    fn start_logview(&mut self) {
        let logs = self.status_log.clone();
        let sr = &self.signal_receiver;
        let mut ot = OverlayText::with_logs(logs, true);
        ot.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));
        self.child_widget = Some((Box::new(ot), OVERLAY_LAYOUT));
    }

    fn start_goto(&mut self) {
        let mut gt = GotoInputLineBehavior::new();
        let sr = &self.signal_receiver;
        gt.on_done.connect(signal!(sr with |obj, pos| {
            obj.child_widget = None;
            obj.goto(pos);
        }));

        gt.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));

        self.child_widget = Some((Box::new(InputLine::new(gt)) as Box<Widget>, INPUTLINE_LAYOUT));
    }

    fn start_find(&mut self) {
        let mut find_line = FindInputLine::new();
        let sr = &self.signal_receiver;
        find_line.on_find.connect(signal!(sr with |obj, needle| {
            obj.child_widget = None;
            obj.find_buf(&needle);
        }));

        find_line.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));

        self.child_widget = Some((Box::new(InputLine::new(find_line)) as Box<Widget>, INPUTLINE_LAYOUT));
    }

    fn start_save(&mut self) {
        let mut path_line: PathInputLine<FS> = PathInputLine::new(PathInputType::Save);
        let sr = &self.signal_receiver;
        path_line.on_done.connect(signal!(sr with |obj, path| {
            obj.child_widget = None;
            obj.save(&path);
        }));

        path_line.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));

        self.child_widget = Some((Box::new(InputLine::new(path_line)) as Box<Widget>, INPUTLINE_LAYOUT));
    }

    fn start_open(&mut self) {
        let mut path_line: PathInputLine<FS> = PathInputLine::new(PathInputType::Open);
        let sr = &self.signal_receiver;
        path_line.on_done.connect(signal!(sr with |obj, path| {
            obj.child_widget = None;
            obj.open(&path);
        }));

        path_line.on_cancel.connect(signal!(sr with |obj, opt_msg| {
            obj.child_widget = None;
            if let Some(ref msg) = opt_msg {
                obj.status(msg.clone());
            } else {
                obj.clear_status();
            }
        }));

        self.child_widget = Some((Box::new(InputLine::new(path_line)) as Box<Widget>, INPUTLINE_LAYOUT));
    }

    fn process_msgs(&mut self) {
        let sr = self.signal_receiver.clone();
        sr.run(self);
    }

    pub fn input(&mut self, key: KeyPress) {
        self.process_msgs();

        if let Some((ref mut child_widget, _)) = self.child_widget {
            child_widget.input(&self.input, key);
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

    pub fn get_position(&mut self) -> isize {
        self.cursor_nibble_pos.to_bytes()
    }

    pub fn get_file_path(&mut self) -> Option<&Path> {
        match self.cur_path {
            Some(ref p) => Some(p.as_path()),
            None => None,
        }
    }

    pub fn get_config(&mut self) -> &Config<FS> {
        &self.config
    }
}
