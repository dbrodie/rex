use std::iter;
use std::cmp;
use util::string_with_repeat;
use rustbox::{RustBox};
use rustbox::keyboard::Key;

use super::common::{Rect, Canceled};
use super::RustBoxEx::{RustBoxEx, Style};
use super::input::Input;

pub enum OverlayActions {
    Cancel,
}

pub struct OverlayText {
    text: String,
    pub on_cancel: Canceled,
}

impl OverlayText {
    pub fn with_text(text: String) -> OverlayText {
        OverlayText {
            text: text,
            on_cancel: Default::default(),
        }
    }

    pub fn input(&mut self, input: &Input, key: Key) -> bool {
        let action = if let Some(action) = input.overlay_input(key) { action } else {
            return false;
        };
        match action {
            OverlayActions::Cancel => {
                self.on_cancel.signal(None);
                true
            }
        }
    }

    pub fn draw(&mut self, rb: &RustBox, area: Rect<isize>, has_focus: bool) {
        let repeat: iter::Repeat<Option<&str>> = iter::repeat(None);
        let iter =
            self.text.lines()
                .map(
                    // Chomp the width of each line
                    |line| Some(&line[0..cmp::min(line.len(), (area.right - area.left) as usize)])
                    // |line| Some(line.slice_to(cmp::min(line.len(), (area.right - area.left) as usize )))
                    // Add "empty lines" - we need this so we clear the screen on empty lines
                    )
                .chain(repeat)
            // Take only as many lines as needed
                .take((area.bottom - area.top) as usize)
            // And count them
                .enumerate();

        for (i, opt_line) in iter {
            // Clean the line

            rb.print_style(area.left as usize, (area.top + i as isize) as usize, Style::Default,
                &string_with_repeat(' ', (area.right - area.left) as usize));

            // And draw the text if there is one
            if let Some(line) = opt_line {
                rb.print_style(area.left as usize, (area.top + i as isize) as usize,
                    Style::Default, line);
            }
        }

        if has_focus {
            rb.set_cursor(0, 0);
        }
    }
}
