pub mod mock_frontend;
pub mod mock_filesystem;
pub mod bytes;

use std::fmt::Debug;

use typenum::uint::Unsigned;
use typenum::consts;

use rex::ui::view::HexEdit;

// Little helper function till Iterator.eq stabalizes
pub fn assert_iter_eq<I, J>(one: I, other: J) where
    I: IntoIterator,
    J: IntoIterator,
    I::Item: PartialEq<J::Item> + Debug,
    J::Item: Debug,
{
    let mut one = one.into_iter();
    let mut other = other.into_iter();

    loop {
        match (one.next(), other.next()) {
            (None, None) => return,
            (Some(x), Some(y)) => assert_eq!(x, y),
            (a @ _, b @ _) => panic!("left is {:?}, right is {:?}", a, b)
        }
    }
}

pub fn generate_vec(size: usize) -> Vec<u8> {
    (0..size).map(|x| (x & 0xff) as u8).collect()
}

pub fn simple_init(size: usize) -> (HexEdit<mock_filesystem::MockFilesystem>,
        mock_frontend::MockFrontend) {
    simple_init_with_vec(generate_vec(size))
}

pub fn simple_init_empty() -> (HexEdit<mock_filesystem::MockFilesystem>, mock_frontend::MockFrontend) {
    simple_init_helper(None)
}

pub fn simple_init_with_vec(vec: Vec<u8>) -> (HexEdit<mock_filesystem::MockFilesystem>,
        mock_frontend::MockFrontend) {
    simple_init_helper(Some(vec))
}

pub fn simple_init_helper<T: Unsigned = consts::U0>(maybe_vec: Option<Vec<u8>>) ->
        (HexEdit<mock_filesystem::MockFilesystem<T>>, mock_frontend::MockFrontend) {
    let mut edit: HexEdit<mock_filesystem::MockFilesystem<T>> = HexEdit::new();
    let mut frontend = mock_frontend::MockFrontend::new();

    if let Some(vec) = maybe_vec {
        edit.open_vec(vec);
    }

    edit.resize(100, 100);
    edit.draw(&mut frontend);
    (edit, frontend)
}
