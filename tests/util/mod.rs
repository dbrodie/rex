pub mod mock_frontend;

use rex::config::Config;
use rex::ui::view::HexEdit;

fn generate_vec(size: usize) -> Vec<u8> {
    (0..size).map(|x| (x & 0xff) as u8).collect()
}

pub fn simple_init(size: usize) -> (HexEdit, mock_frontend::MockFrontend) {
    simple_init_with_vec(generate_vec(size))
}

pub fn simple_init_with_vec(vec: Vec<u8>) -> (HexEdit, mock_frontend::MockFrontend) {
    let config : Config = Default::default();

    let mut edit = HexEdit::new(config);
    let mut frontend = mock_frontend::MockFrontend::new();

    edit.open_vec(vec);

    edit.resize(100, 100);
    edit.draw(&mut frontend);
    (edit, frontend)
}
