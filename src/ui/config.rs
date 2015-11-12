use std::default::Default;
use std::rc::Rc;
use std::cmp;

use rex_utils;
use rex_utils::rect::Rect;

use super::super::config::{Config, Value};
use super::common::Canceled;
use super::input::Input;
use super::widget::Widget;
use super::super::frontend::{Frontend, Style, KeyPress};

pub enum ConfigScreenActions {
    Up,
    Down,
    Select,
    Cancel,
}

signal_decl!{ConfigSelected(&'static str, Value)}

pub struct ConfigScreen {
    pub on_cancel: Canceled,
    pub on_selected: ConfigSelected,
    config: Rc<Config>,
    cursor_line: isize,
}

impl ConfigScreen {
    pub fn with_config(config: Rc<Config>) -> ConfigScreen {
        ConfigScreen {
            on_cancel: Default::default(),
            on_selected: Default::default(),
            config: config,
            cursor_line: 0,
        }
    }

    fn select(&mut self) {
        if let Some((name, val)) = self.config.values().nth(self.cursor_line as usize) {
            self.on_selected.signal(name, val);
        }
    }
}

impl Widget for ConfigScreen {
    fn input(&mut self, input: &Input, key: KeyPress) -> bool {
        let action = if let Some(action) = input.config_input(key) { action } else {
            return false;
        };

        match action {
            ConfigScreenActions::Down => { self.cursor_line = self.cursor_line + 1; }
            ConfigScreenActions::Up =>  { self.cursor_line = cmp::max(0, self.cursor_line - 1); }
            ConfigScreenActions::Select => (self.select()),
            ConfigScreenActions::Cancel => { self.on_cancel.signal(None); }
        };
        return true;
    }

    fn draw(&mut self, rb: &mut Frontend, area: Rect<isize>, _: bool) {
        rb.set_cursor(-1, -1);
        let clear_line = rex_utils::string_with_repeat(' ', area.width as usize);

        for i in 0..(area.height as usize) {
            rb.print_style(area.left as usize, area.top as usize + i, Style::Default, &clear_line);
        }

        for (i, (name, value)) in self.config.values().enumerate() {
            let style = if i != self.cursor_line as usize {
                Style::Default
            } else {
                Style::Selection
            };
            rb.print_style(area.left as usize, area.top as usize + i, style,
                &format!("{} = {}", name, value));
        }
    }
}
