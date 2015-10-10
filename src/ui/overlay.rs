use std::iter;
use std::cmp;
use std::str::Lines;
use std::slice::Iter;

use rex_utils;
use rex_utils::iter_optional::IterOptionalExt;
use rex_utils::rect::Rect;

use super::common::Canceled;
use super::super::frontend::{Frontend, Style, KeyPress};
use super::input::Input;
use super::widget::Widget;

enum ToLinesIter<'a> {
    StringLines(Lines<'a>),
    SliceLines(Iter<'a, String>)
}

pub trait ToLines {
    fn to_lines<'a>(&'a self) -> ToLinesIter<'a>;
}

impl ToLines for String {
    fn to_lines<'a>(&'a self) -> ToLinesIter<'a> {
        ToLinesIter::StringLines(self.lines())
    }
}

impl ToLines for Vec<String> {
    fn to_lines<'a>(&'a self) -> ToLinesIter<'a> {
        ToLinesIter::SliceLines(self.iter())
    }
}

impl<'a> Iterator for ToLinesIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        match *self {
            ToLinesIter::StringLines(ref mut lines) => lines.next(),
            ToLinesIter::SliceLines(ref mut lines) => lines.next().map(|x| &x[..]),
        }
    }
}

impl<'a> DoubleEndedIterator for ToLinesIter<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        match *self {
            ToLinesIter::StringLines(ref mut lines) => lines.next_back(),
            ToLinesIter::SliceLines(ref mut lines) => lines.next_back().map(|x| &x[..]),
        }
    }
}

pub enum OverlayActions {
    Cancel,
}

pub struct OverlayText {
    text: Box<ToLines>,
    reverse: bool,
    pub on_cancel: Canceled,
}

impl OverlayText {
    pub fn with_text(text: String, rev: bool) -> OverlayText {
        OverlayText {
            text: Box::new(text),
            reverse: rev,
            on_cancel: Default::default(),
        }
    }

    pub fn with_logs(text: Vec<String>, rev: bool) -> OverlayText {
        OverlayText {
            text: Box::new(text),
            reverse: rev,
            on_cancel: Default::default(),
        }
    }
}

impl Widget for OverlayText {
    fn input(&mut self, input: &Input, key: KeyPress) -> bool {
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

    fn draw(&mut self, rb: &Frontend, area: Rect<isize>, has_focus: bool) {
        let repeat: iter::Repeat<Option<&str>> = iter::repeat(None);
        let iter = self.text.to_lines().optional(self.reverse, |it| it.rev(), |it| it).map(
                    // Chomp the width of each line
                    |line| Some(&line[0..cmp::min(line.len(), (area.width) as usize)])
                    )
                .chain(repeat)
            // Take only as many lines as needed
                .take((area.height) as usize)
            // And count them
                .enumerate();

        for (i, opt_line) in iter {
            // Clean the line

            rb.print_style(area.left as usize, (area.top + i as isize) as usize, Style::Default,
                &rex_utils::string_with_repeat(' ', (area.width) as usize));

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
