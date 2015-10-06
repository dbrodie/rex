#[derive(Copy, Clone)]
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

pub trait Frontend {
    fn clear(&self);
    fn present(&self);
    fn print_style(&self, x: usize, y: usize, style: Style, s: &str);
    fn print_char_style(&self, x: usize, y: usize, style: Style, c: char);
    fn print_slice_style(&self, x: usize, y: usize, style: Style, chars: &[char]);
    fn set_cursor(&self, x: isize, y: isize);
    fn height(&self) -> usize;
    fn width(&self) -> usize;
}
