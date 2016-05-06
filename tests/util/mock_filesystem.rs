use std::path::{Path, PathBuf};
use std::io;
use std::str;
use std::io::{Cursor, Read, Write};
use std::ops::DerefMut;
use std::collections::hash_map::{HashMap, Entry};
use std::sync::{Arc, Mutex};
use std::mem;
use std::marker::PhantomData;

use rex::filesystem::Filesystem;

const CONFIG_PATH: &'static str = "/config/rex/rex.conf";


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

/// A backend for the mock filesystem, providing access to the actual filesystem data needed to
/// implement the filesystem. This allows it to be saved in a per-thread or global basis, for
/// example.
pub trait MockFilesystemBackend {
    /// Returns the file system implementation data, whereever it may be saved.
    fn get_backend() -> MockFilesystemImpl;
}

struct MockFilesystemImpl {
    files: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<Vec<u8>>>>>>,
}

impl Default for MockFilesystemImpl {
    fn default() -> MockFilesystemImpl {
        MockFilesystemImpl {
            files: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}

impl Clone for MockFilesystemImpl {
    fn clone(&self) -> MockFilesystemImpl {
        MockFilesystemImpl {
            files: self.files.clone()
        }
    }
}

/// A mock filesystem, providing basic filesystem APIs, the backend data storage is generic.
pub struct MockFilesystem<T: MockFilesystemBackend + 'static = ThreadLocalMockFilesystemBackend>(PhantomData<T>);

#[derive(Debug, Clone, Copy)]
struct ThreadLocalMockFilesystemBackend;

/// A mock filesystem that saves seperate data per thread.
pub type ThreadedMockFilesystem = MockFilesystem<ThreadLocalMockFilesystemBackend>;

thread_local!(static THREAD_LOCAL_MOCK_FILESYSTEM: MockFilesystemImpl = Default::default());

impl MockFilesystemBackend for ThreadLocalMockFilesystemBackend {
    fn get_backend() -> MockFilesystemImpl {
        let mut ret: Option<MockFilesystemImpl> = None;
        THREAD_LOCAL_MOCK_FILESYSTEM.with(
            |val| ret = Some(val.clone())
        );
        ret.unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
struct GlobalMockFilesystemBackend;

/// A mock filesystem that saves seperate data per thread.
pub type GlobalMockFilesystem = MockFilesystem<GlobalMockFilesystemBackend>;

lazy_static!{
    static ref GLOBAL_MOCK_FILESYSTEM: MockFilesystemImpl = Default::default();
}

impl MockFilesystemBackend for GlobalMockFilesystemBackend {
    fn get_backend() -> MockFilesystemImpl {
        GLOBAL_MOCK_FILESYSTEM.clone()
    }
}

impl<T: MockFilesystemBackend + 'static> Filesystem for MockFilesystem<T> {
    type FSRead = MockFile;
    type FSWrite = MockFile;

    fn open_config<P1: AsRef<Path>, P2: AsRef<Path>>(prefix: P1, config_name: P2) -> Option<PathBuf> {
        Some(PathBuf::from(CONFIG_PATH))
    }
    
    fn save_config<P1: AsRef<Path>, P2: AsRef<Path>>(prefix: P1, config_name: P2) -> io::Result<PathBuf> {
        Ok(PathBuf::from(CONFIG_PATH))
    }

    fn make_absolute<P: AsRef<Path>>(p: P) -> io::Result<PathBuf> {
        Ok(p.as_ref().into())
    }

    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self::FSRead> {
        let backend = T::get_backend();
        let file_map = backend.files.lock().unwrap();
        file_map.get(path.as_ref()).ok_or(io::Error::new(io::ErrorKind::NotFound, "File not found!")).map(|file|
            MockFile::new(file.clone())
        )
    }

    fn can_open<P: AsRef<Path>>(_p: P) -> io::Result<()> {
        Ok(())
    }

    fn save<P: AsRef<Path>>(path: P) -> io::Result<Self::FSWrite> {
        let backend = T::get_backend();
        let mut file_map = backend.files.lock().unwrap();
        let file = file_map.entry(path.as_ref().into()).or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

        Ok(MockFile::new(file.clone()))
    }

    fn can_save<P: AsRef<Path>>(_p: P) -> io::Result<()> {
        Ok(())
    }
}

impl<T: MockFilesystemBackend + 'static> MockFilesystem<T> {
    pub fn open_config() -> io::Result<<MockFilesystem<T> as Filesystem>::FSRead> {
        Self::open(CONFIG_PATH)
    }

    pub fn save_config() -> io::Result<<MockFilesystem<T> as Filesystem>::FSWrite> {
        Self::save(CONFIG_PATH)
    }

    pub fn list_paths() -> Vec<PathBuf> {
        let backend = T::get_backend();
        let r : Vec<PathBuf> = backend.files.lock().unwrap().keys().map( |i| i.clone() ).collect();
        r
    }

    pub fn reset() {
        let backend = T::get_backend();
        backend.files.lock().unwrap().clear();
    }

    pub fn get_inner<'a, P: AsRef<Path>>(path: P) -> Vec<u8> {
        // This function is very ugly, in general we would like to "unwrap" the file from the
        // mock filesystem. Sadly, there doesn't seem to be a better way.
        let backend = T::get_backend();
        let mut file_map = backend.files.lock().unwrap();
        let f = file_map.remove(path.as_ref()).unwrap();
        let a = Arc::try_unwrap(f).unwrap();
        let m = a.lock().unwrap();
        let v = m.clone();
        v
    }

    pub fn put<'a, P: AsRef<Path>>(path: P, v: Vec<u8>) {
        let backend = T::get_backend();
        let mut file_map = backend.files.lock().unwrap();
        file_map.insert(path.as_ref().into(), Arc::new(Mutex::new(v)));
    }
}
