use std::iter;
use std::cmp;
use util::string_with_repeat;
use rustbox::{RustBox, Color, RB_NORMAL, RB_BOLD};

use super::common::{Rect, Canceled};

pub struct OverlayText {
    text: String,
    offset: isize,
    pub on_cancel: Canceled,
}

impl OverlayText {
    pub fn with_text(text: String) -> OverlayText {
        OverlayText {
            text: text,
            offset: 0,
            on_cancel: Default::default(),
        }
    }

    pub fn input(&mut self, emod: u8, key: u16, ch: u32) -> bool {
        match (emod, key, ch) {
            (0, 0, 113) => {
                self.on_cancel.signal(None);
                true
            }
            _ => false
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

            rb.print(area.left as usize, (area.top + i as isize) as usize, RB_NORMAL, Color::White,
                     Color::Black, &string_with_repeat(' ', (area.right - area.left) as usize));

            // And draw the text if there is one
            match opt_line {
                Some(line) => {
                    rb.print(area.left as usize, (area.top + i as isize) as usize, RB_NORMAL,
                             Color::White, Color::Black, line);
                }
                None => ()
            }
        }

        if has_focus {
            rb.set_cursor(0, 0);
        }
    }
}
