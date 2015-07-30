use rustbox::keyboard::Key;

use super::view::HexEditActions;
use super::inputline::BaseInputLineActions;
use super::overlay::OverlayActions;

pub struct Input;

impl Input {
    pub fn new() -> Input {
        Input
    }
    pub fn editor_input(&self, key: Key) -> Option<HexEditActions> {
        match key {
            Key::Char(c) => Some(HexEditActions::Edit(c)),
            Key::Left => Some(HexEditActions::MoveLeft),
            Key::Right => Some(HexEditActions::MoveRight),
            Key::Up => Some(HexEditActions::MoveUp),
            Key::Down => Some(HexEditActions::MoveDown),
            Key::PageUp => Some(HexEditActions::MovePageUp),
            Key::PageDown => Some(HexEditActions::MovePageDown),
            Key::Backspace => Some(HexEditActions::DeleteWithMove),
            Key::Delete => Some(HexEditActions::Delete),
            Key::Tab => Some(HexEditActions::SwitchView),
            Key::Ctrl('x') => Some(HexEditActions::CutSelection),
            Key::Ctrl('c') => Some(HexEditActions::CopySelection),
            Key::Ctrl('v') => Some(HexEditActions::PasteSelection),
            Key::Ctrl('/') => Some(HexEditActions::HelpView),
            Key::Ctrl('l') => Some(HexEditActions::LogView),
            Key::Ctrl('o') => Some(HexEditActions::ToggleInsert),
            Key::Ctrl('s') => Some(HexEditActions::ToggleSelecion),
            Key::Ctrl('z') => Some(HexEditActions::Undo),
            Key::Ctrl('g') => Some(HexEditActions::AskGoto),
            Key::Ctrl('f') => Some(HexEditActions::AskFind),
            Key::Ctrl('e') => Some(HexEditActions::AskOpen),
            Key::Ctrl('w') => Some(HexEditActions::AskSave),

            k @ _=> {
                println!("Unknown key {:?}", k);
                None
            }
        }
    }

    pub fn inputline_input(&self, key: Key) -> Option<BaseInputLineActions> {
        match key {
            Key::Char(c) => Some(BaseInputLineActions::Edit(c)),
            Key::Ctrl(c) => Some(BaseInputLineActions::Ctrl(c)),
            Key::Left => Some(BaseInputLineActions::MoveLeft),
            Key::Right => Some(BaseInputLineActions::MoveRight),
            Key::Backspace => Some(BaseInputLineActions::DeleteWithMove),
            Key::Enter => Some(BaseInputLineActions::Ok),
            Key::Esc => Some(BaseInputLineActions::Cancel),
            _ => None
        }

    }

    pub fn overlay_input(&self, key: Key) -> Option<OverlayActions> {
        match key {
            Key::Esc => Some(OverlayActions::Cancel),
            _ => None
        }
    }
}
