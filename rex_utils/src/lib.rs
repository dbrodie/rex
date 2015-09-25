//! rex_utils is a small side library to allow to seperate the non-core parts out of rex.
//!
//! This allows us to have proper unit tests to the crate (since unit tests for binary crates in
//! rust are not well supported).
extern crate itertools;

use std::iter;
use std::char;

pub mod iter_optional;
pub mod split_vec;
pub mod rect;
pub mod relative_rect;
#[macro_use] pub mod signals;

/// Create a string with a repeated character.
///
/// # Examples
///
/// ```
/// use rex_utils::string_with_repeat;
///
/// assert_eq!(string_with_repeat('a', 5), "aaaaa");
/// ```
pub fn string_with_repeat(c: char, n: usize) -> String {
    let v: Vec<_> = iter::repeat(c as u8).take(n).collect();
    String::from_utf8(v).unwrap()
}

/// Checks if num is between the "normalized" range a and b. Normalized,
/// meaning, that if b is larger than a, or vice versa, the right test is done.
///
/// # Examples
/// ```
/// use rex_utils::is_between;
///
/// assert!(is_between(2, 1, 5));
/// assert!(is_between(2, 5, 1));
/// assert!(!is_between(5, 1, 2));
/// assert!(!is_between(1, 5, 2));
/// ```
pub fn is_between<N: PartialOrd>(num: N, a: N, b: N) -> bool {
    let (smaller, larger) = if a < b { (a, b) } else { (b, a) };
    (smaller <= num) && (num <= larger)
}

/// Converts a byte to its hex representation
///
/// # Return value
///
/// Returns a tuple of the two characters required for the representation, in lowercase.
///
/// # Examples
///
/// ```
/// use rex_utils;
///
/// assert_eq!(rex_utils::u8_to_hex(0xab), ('a', 'b'));
/// ```
pub fn u8_to_hex(b: u8) -> (char, char) {
    (char::from_digit((b as u32 >> 4) & 0xF, 16).unwrap(),
    char::from_digit(b as u32 & 0xF, 16).unwrap())
}
