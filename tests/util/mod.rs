pub mod mock_frontend;
pub mod mock_filesystem;
pub mod bytes;

use rex::config::Config;
use rex::ui::view::HexEdit;

pub fn generate_vec(size: usize) -> Vec<u8> {
    (0..size).map(|x| (x & 0xff) as u8).collect()
}

pub fn simple_init(size: usize) -> (HexEdit<mock_filesystem::MockFilesystem>, mock_frontend::MockFrontend) {
    simple_init_with_vec(generate_vec(size))
}

pub fn simple_init_empty() -> (HexEdit<mock_filesystem::MockFilesystem>, mock_frontend::MockFrontend) {
    simple_init_helper(None)
}

pub fn simple_init_with_vec(vec: Vec<u8>) -> (HexEdit<mock_filesystem::MockFilesystem>, mock_frontend::MockFrontend) {
    simple_init_helper(Some(vec))
}

fn simple_init_helper(maybe_vec: Option<Vec<u8>>) -> (HexEdit<mock_filesystem::MockFilesystem>, mock_frontend::MockFrontend) {
    mock_filesystem::MockFilesystem::reset();

    let config : Config = Default::default();

    let mut edit: HexEdit<mock_filesystem::MockFilesystem> = HexEdit::new(config);
    let mut frontend = mock_frontend::MockFrontend::new();

    if let Some(vec) = maybe_vec {
        edit.open_vec(vec);
    }

    edit.resize(100, 100);
    edit.draw(&mut frontend);
    (edit, frontend)
}
