#[derive(Copy, Clone, Debug)]
pub enum Style {
    Default,
    Selection,
    Hint,
    StatusBar,
    InputLine,
    MenuShortcut,
    MenuEntry,
    MenuTitle
}

#[derive(Copy, Clone, Debug)]
pub enum KeyPress {
    Key(char),
    Shortcut(char),
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Backspace,
    Delete,
    Tab,
    Insert,
    Enter,
    Esc
}

pub enum Event {
    KeyPressEvent(KeyPress),
    Resize(usize, usize),
}

pub trait Frontend {
    fn clear(&self);
    fn present(&self);
    fn print_style(&self, x: usize, y: usize, style: Style, s: &str);
    fn print_char_style(&self, x: usize, y: usize, style: Style, c: char);
    fn print_slice_style(&self, x: usize, y: usize, style: Style, chars: &[char]);
    fn set_cursor(&mut self, x: isize, y: isize);
    fn height(&self) -> usize;
    fn width(&self) -> usize;
    fn poll_event(&mut self) -> Event;
}
