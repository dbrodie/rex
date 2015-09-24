use std::default::Default;
use std::rc::Rc;
use std::cmp;
use rustbox::RustBox;
use rustbox::keyboard::Key;

use rex_utils;
use rex_utils::rect::Rect;

use super::super::config::Config;
use super::common::Canceled;
use super::input::Input;
use super::widget::Widget;
use super::view::HexEditActions;
use super::RustBoxEx::{RustBoxEx, Style};

pub enum ConfigScreenActions {
    Up,
    Down,
    Select,
    Cancel,
}

pub enum MenuEntry<'a, T> where T: 'a {
    SubEntries(char, &'a str, &'a [MenuEntry<'a, T>]),
    CommandEntry(char, &'a str, T)
}

pub type MenuState<T> = &'static [MenuEntry<'static, T>];

pub struct ConfigScreen {
    pub on_cancel: Canceled,
    config: Rc<Config>,
    cursor_line: isize,
}

impl ConfigScreen {
    pub fn with_config(config: Rc<Config>) -> ConfigScreen {
        ConfigScreen {
            on_cancel: Default::default(),
            config: config,
            cursor_line: 0,
        }
    }
}

impl Widget for ConfigScreen {
    fn input(&mut self, input: &Input, key: Key) -> bool {
        let action = if let Some(action) = input.config_input(key) { action } else {
            return false;
        };

        match action {
            ConfigScreenActions::Down => { self.cursor_line = self.cursor_line + 1; }
            ConfigScreenActions::Up =>  { self.cursor_line = cmp::max(0, self.cursor_line - 1); }
            ConfigScreenActions::Select => (),
            ConfigScreenActions::Cancel => { self.on_cancel.signal(None); }
        };
        return true;
    }

    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool) {
        let clear_line = rex_utils::string_with_repeat(' ', area.width as usize);

        for i in 0..(area.height as usize) {
            rb.print_style(area.left as usize, area.top as usize + i, Style::Default, &clear_line);
        }

        for (i, (name, value)) in self.config.values().enumerate() {
            rb.print_style(area.left as usize, area.top as usize + i, Style::Default,
                &format!("{} = {}", name, value));
        }
    }
}
