// use std::slice;
use std::path::{Path, PathBuf};
use std::io;
use std::io::{Cursor, Read, Write};
use std::ops::DerefMut;
use std::collections::hash_map::{HashMap, Entry};
use std::sync::{Arc, Mutex, MutexGuard};
use std::cmp;
use std::mem;
use std::marker::PhantomData;
use typenum::uint::Unsigned;
use typenum::consts;

use rex::filesystem::Filesystem;

use super::bytes;

pub type DefaultConfig = consts::U0;
pub type TestOpenSaveConfig = consts::U1;
const NumConfigTests: usize = 2;

const CONFIG_PATH: &'static str = "/config/rex/rex.conf";

lazy_static! {
    static ref FILES: Mutex<HashMap<PathBuf, Arc<Mutex<Vec<u8>>>>> = Mutex::new(HashMap::new());
    static ref CONFIG_FILES: Mutex<[Option<Arc<Mutex<Vec<u8>>>>; NumConfigTests]> = {
        let mut tmp: [Option<Arc<Mutex<Vec<u8>>>>; NumConfigTests] = [
            None,
            None
        ];
        Mutex::new(tmp)
    };
}

pub struct MockFile(Arc<Mutex<Vec<u8>>>, u64);

impl MockFile {
    fn new(vec: Arc<Mutex<Vec<u8>>>) -> MockFile {
        MockFile(vec, 0)
    }
}

// A small wrapper to help with a workaround until https://github.com/rust-lang/rust/issues/30132
// is fixed.
macro_rules! do_with_cursor {
    ($obj:expr, $func:ident($($arg:expr),*)) => ({
        let mut self_ = $obj;
        let mut vec = self_.0.lock().unwrap();
        let mut p_vec = vec.deref_mut();
        let mut tmp = Vec::new();

        mem::swap(p_vec, &mut tmp);
        let mut c = Cursor::new(tmp);
        c.set_position(self_.1);
        let ret = c.$func($($arg),*);
        self_.1 = c.position();
        tmp = c.into_inner();
        mem::swap(p_vec, &mut tmp);
        ret
    })
}

impl Read for MockFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        do_with_cursor!(self, read(buf))
    }
}

impl Write for MockFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        do_with_cursor!(self, write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        do_with_cursor!(self, flush())
    }
}

/// A mock implementation of `Filesystem` over thread-safe buffers
pub struct MockFilesystem<N: Unsigned = consts::U0> (
    PhantomData<N>
);

pub type DefMockFilesystem = MockFilesystem<consts::U0>;


impl<N: Unsigned> Filesystem for MockFilesystem<N> {
    type FSRead = MockFile;
    type FSWrite = MockFile;

    fn get_config_home() -> PathBuf {
        PathBuf::from("/config/")
    }

    fn make_absolute<P: AsRef<Path>>(p: P) -> io::Result<PathBuf> {
        Ok(p.as_ref().into())
    }

    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self::FSRead> {
        if path.as_ref() == Path::new(CONFIG_PATH) {
            return Self::open_config()
        }
        FILES.lock().unwrap().get(path.as_ref()).ok_or(io::Error::new(io::ErrorKind::NotFound, "File not found!")).map(|file|
            MockFile::new(file.clone())
        )
    }

    fn can_open<P: AsRef<Path>>(p: P) -> io::Result<()> {
        Ok(())
    }

    fn save<P: AsRef<Path>>(path: P) -> io::Result<Self::FSWrite> {
        if path.as_ref() == Path::new(CONFIG_PATH) {
            return Self::save_config()
        }
        let file = Arc::new(Mutex::new(Vec::new()));
        if let Entry::Vacant(entry) = FILES.lock().unwrap().entry(path.as_ref().into()) {
            entry.insert(file.clone());
            Ok(MockFile::new(file))
        } else {
            Err(io::Error::new(io::ErrorKind::AlreadyExists, "File alredy exists!"))
        }
    }

    fn can_save<P: AsRef<Path>>(p: P) -> io::Result<()> {
        Ok(())
    }
}

impl<N: Unsigned> MockFilesystem<N> {
    pub fn open_config() -> io::Result<<MockFilesystem<N> as Filesystem>::FSRead> {
        if let Some(ref file) = CONFIG_FILES.lock().unwrap()[N::to_usize()] {
            Ok(MockFile::new(file.clone()))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found!"))
        }
    }

    pub fn save_config() -> io::Result<<MockFilesystem<N> as Filesystem>::FSWrite> {
        let mut configs = CONFIG_FILES.lock().unwrap();
        if let Some(ref file) = configs[N::to_usize()] {
            return Ok(MockFile::new(file.clone()));
        }

        let file = Arc::new(Mutex::new(Vec::new()));
        configs[N::to_usize()] = Some(file.clone());
        Ok(MockFile::new(file))
    }

    pub fn reset() {
        FILES.lock().unwrap().clear();
    }

    pub fn get_inner<'a, P: AsRef<Path>>(path: P) -> Vec<u8> {
        // This function is very ugly, in general we would like to "unwrap" the file from the
        // mock filesystem. Sadly, there doesn't seem to be a better way.
        let f = FILES.lock().unwrap().remove(path.as_ref()).unwrap();
        let a = Arc::try_unwrap(f).unwrap();
        let m = a.lock().unwrap();
        let v = m.clone();
        v
    }

    pub fn put<'a, P: AsRef<Path>>(path: P, v: Vec<u8>) {
        FILES.lock().unwrap().insert(path.as_ref().into(), Arc::new(Mutex::new(v)));
    }
}
