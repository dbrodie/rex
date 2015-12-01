use super::super::frontend::KeyPress;
use super::view::HexEditActions;
use super::inputline::BaseInputLineActions;
use super::overlay::OverlayActions;
use super::menu::MenuActions;
use super::configscreen::ConfigScreenActions;

pub struct Input;

impl Input {
    pub fn new() -> Input {
        Input
    }
    pub fn editor_input(&self, key: KeyPress) -> Option<HexEditActions> {
        match key {
            KeyPress::Left => Some(HexEditActions::MoveLeft),
            KeyPress::Right => Some(HexEditActions::MoveRight),
            KeyPress::Up => Some(HexEditActions::MoveUp),
            KeyPress::Down => Some(HexEditActions::MoveDown),
            KeyPress::PageUp => Some(HexEditActions::MovePageUp),
            KeyPress::PageDown => Some(HexEditActions::MovePageDown),
            KeyPress::Home => Some(HexEditActions::MoveToFirstColumn),
            KeyPress::End => Some(HexEditActions::MoveToLastColumn),
            KeyPress::Backspace => Some(HexEditActions::DeleteWithMove),
            KeyPress::Delete => Some(HexEditActions::Delete),
            KeyPress::Tab => Some(HexEditActions::SwitchView),
            KeyPress::Insert => Some(HexEditActions::ToggleInsert),
            KeyPress::Shortcut(' ') => Some(HexEditActions::ToggleSelecion),
            KeyPress::Shortcut('x') => Some(HexEditActions::CutSelection),
            KeyPress::Shortcut('c') => Some(HexEditActions::CopySelection),
            KeyPress::Shortcut('v') => Some(HexEditActions::PasteSelection),
            KeyPress::Shortcut('/') => Some(HexEditActions::HelpView),
            KeyPress::Shortcut('l') => Some(HexEditActions::LogView),
            KeyPress::Shortcut('z') => Some(HexEditActions::Undo),
            KeyPress::Shortcut('g') => Some(HexEditActions::AskGoto),
            KeyPress::Shortcut('f') => Some(HexEditActions::AskFind),
            KeyPress::Shortcut('o') => Some(HexEditActions::AskOpen),
            KeyPress::Shortcut('s') => Some(HexEditActions::AskSave),
            KeyPress::Shortcut('\\') => Some(HexEditActions::StartMenu),
            KeyPress::Key(c) => Some(HexEditActions::Edit(c)),

            k @ _ => {
                println!("Unknown key {:?}", k);
                None
            }
        }
    }

    pub fn inputline_input(&self, key: KeyPress) -> Option<BaseInputLineActions> {
        match key {
            KeyPress::Key(c) => Some(BaseInputLineActions::Edit(c)),
            KeyPress::Shortcut(c) => Some(BaseInputLineActions::Ctrl(c)),
            KeyPress::Left => Some(BaseInputLineActions::MoveLeft),
            KeyPress::Right => Some(BaseInputLineActions::MoveRight),
            KeyPress::Delete => Some(BaseInputLineActions::Delete),
            KeyPress::Backspace => Some(BaseInputLineActions::DeleteWithMove),
            KeyPress::Enter => Some(BaseInputLineActions::Ok),
            KeyPress::Esc => Some(BaseInputLineActions::Cancel),
            _ => None
        }

    }

    pub fn overlay_input(&self, key: KeyPress) -> Option<OverlayActions> {
        match key {
            KeyPress::Esc => Some(OverlayActions::Cancel),
            _ => None
        }
    }

    pub fn config_input(&self, key: KeyPress) -> Option<ConfigScreenActions> {
        match key {
            KeyPress::Down => Some(ConfigScreenActions::Down),
            KeyPress::Up => Some(ConfigScreenActions::Up),
            KeyPress::Enter => Some(ConfigScreenActions::Select),
            KeyPress::Esc => Some(ConfigScreenActions::Cancel),
            _ => None
        }
    }

    pub fn menu_input(&self, key: KeyPress) -> Option<MenuActions> {
        match key {
            KeyPress::Backspace => Some(MenuActions::Back),
            KeyPress::Esc => Some(MenuActions::Cancel),
            KeyPress::Key('?') => Some(MenuActions::ToggleHelp),
            KeyPress::Key(c) => Some(MenuActions::Key(c)),
            _ => None
        }
    }
}
