use std::default::Default;
use rustbox::RustBox;
use rustbox::keyboard::Key;

use rex_utils;
use rex_utils::rect::Rect;

use super::common::Canceled;
use super::input::Input;
use super::widget::Widget;
use super::view::HexEditActions;
use super::RustBoxEx::{RustBoxEx, Style};

pub enum MenuActions {
    Key(char),
    Back,
    Cancel,
    ToggleHelp,
}

pub enum MenuEntry<'a, T> where T: 'a {
    SubEntries(char, &'a str, &'a [MenuEntry<'a, T>]),
    CommandEntry(char, &'a str, T)
}

pub type MenuState<T> = &'static [MenuEntry<'static, T>];

signal_decl!{MenuSelected(HexEditActions)}

pub struct OverlayMenu {
    root_menu: MenuState<HexEditActions>,
    menu_stack: Vec<&'static MenuEntry<'static, HexEditActions>>,
    show_help: bool,
    pub on_cancel: Canceled,
    pub on_selected: MenuSelected,
}

impl OverlayMenu {
    pub fn with_menu(root_menu: MenuState<HexEditActions>) -> OverlayMenu {
        OverlayMenu {
            root_menu: root_menu,
            menu_stack: vec![],
            show_help: false,
            on_cancel: Default::default(),
            on_selected: Default::default(),
        }
    }

    fn menu_act_key(&mut self, c: char) -> bool {
        for entry in self.current_menu().iter() {
            match entry {
                &MenuEntry::CommandEntry(key, _, command) if key == c => {
                    self.on_selected.signal(command);
                    return true;
                }
                &MenuEntry::SubEntries(key, _, sub_menu) if key == c => {
                    self.menu_stack.push(&entry);
                    return true;
                }
                _ => ()
            }
        }
        return false;
    }

    fn current_menu(&self) -> MenuState<HexEditActions> {
        match self.menu_stack.last() {
            Some(&&MenuEntry::SubEntries(_, _, entries)) => entries,
            Some(&&MenuEntry::CommandEntry(_, _, _)) => panic!("Got a non menu entry in my menu stack!"),
            None => &self.root_menu,
        }
    }

    fn menu_back(&mut self) {
        self.menu_stack.pop();
    }

    fn draw_menu_location(&mut self, rb: &RustBox, x: isize, y: isize) {
        let mut x_pos = 0;
        rb.print_style((x + x_pos) as usize, y as usize, Style::MenuTitle, " > ");
        x_pos += 3;
        for location_entry in self.menu_stack.iter() {
            let name = match **location_entry {
                MenuEntry::SubEntries(_, name, _) => name,
                _ => panic!("Got a non menu entry in my menu stack!")
            };
            rb.print_style((x + x_pos) as usize, y as usize, Style::MenuTitle, name);
            x_pos += name.len() as isize;
            rb.print_style((x + x_pos) as usize, y as usize, Style::MenuTitle, " > ");
            x_pos += 3;

        }
    }
}

impl Widget for OverlayMenu {
    fn input(&mut self, input: &Input, key: Key) -> bool {
        let action = if let Some(action) = input.menu_input(key) { action } else {
            return false;
        };

        match action {
            MenuActions::Back => self.menu_back(),
            MenuActions::Key(c) => { return self.menu_act_key(c); }
            MenuActions::Cancel => self.on_cancel.signal(None),
            MenuActions::ToggleHelp => self.show_help = !self.show_help,
        };
        return true;
    }

    fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool) {
        let clear_line = rex_utils::string_with_repeat(' ', area.width as usize);

        if (!self.show_help) {
            let hint_msg = "Use ? to show/hide the menu help";

            rb.print_style(area.left as usize, (area.bottom() - 1) as usize, Style::Default, &clear_line);
            rb.print_style(area.right() as usize - hint_msg.len(), (area.bottom() - 1) as usize,
                Style::Hint, hint_msg);
            self.draw_menu_location(rb, area.left, area.bottom() - 1);
            return;
        }

        for i in 0..(area.height as usize) {
            rb.print_style(area.left as usize, area.top as usize + i, Style::Default, &clear_line);
        }

        self.draw_menu_location(rb, area.left, area.top);

        for (i, entry) in self.current_menu().iter().enumerate() {
            let (key, name, is_title, style) = match entry {
                &MenuEntry::CommandEntry(key, name, _) => (key, name, false, Style::MenuEntry),
                &MenuEntry::SubEntries(key, name, _) => (key, name, true, Style::MenuTitle),
            };
            rb.print_slice_style(10 + area.left as usize, area.top as usize + i + 1, Style::MenuShortcut, &[key, ' ']);
            rb.print_style(10 + area.left as usize + 2, area.top as usize + i + 1, style, name);
            if is_title {
                rb.print_style(10 + area.left as usize + 2 + name.len() + 1, area.top as usize + i + 1, style, "->");
            }
        }

        rb.set_cursor(-1, -1);
    }
}
