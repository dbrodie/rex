// use std::slice;
use std::path::{Path, PathBuf};
use std::io;
use std::io::{Cursor, Read, Write};
use std::collections::hash_map::{HashMap, Entry};
use std::sync::{Arc, Mutex, MutexGuard};
use std::cmp;

use rex::filesystem::Filesystem;

use super::bytes;

lazy_static! {
    pub static ref FILES: Mutex<HashMap<PathBuf, Arc<Mutex<Cursor<Vec<u8>>>>>> = Mutex::new(HashMap::new());
}

pub struct MockFile(Arc<Mutex<Cursor<Vec<u8>>>>);

impl MockFile {
    fn new(vec: Arc<Mutex<Cursor<Vec<u8>>>>) -> MockFile {
        MockFile(vec)
    }
}

impl Read for MockFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut vec = self.0.lock().unwrap();
        vec.read(buf)
    }
}

impl Write for MockFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut vec = self.0.lock().unwrap();
        vec.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut vec = self.0.lock().unwrap();
        vec.flush()
    }
}

pub struct MockFilesystem;


impl Filesystem for MockFilesystem {
    type FSRead = MockFile;
    type FSWrite = MockFile;

    fn make_absolute<P: AsRef<Path>>(p: P) -> io::Result<PathBuf> {
        Ok(p.as_ref().into())
    }

    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self::FSRead> {
        FILES.lock().unwrap().get(path.as_ref()).ok_or(io::Error::new(io::ErrorKind::NotFound, "File not found!")).map(|file|
            MockFile::new(file.clone())
        )
    }

    fn can_open<P: AsRef<Path>>(p: P) -> io::Result<()> {
        Ok(())
    }

    fn save<P: AsRef<Path>>(path: P) -> io::Result<Self::FSWrite> {
        let file = Arc::new(Mutex::new(Cursor::new(Vec::new())));
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

impl MockFilesystem {
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
        v.into_inner()
    }

    pub fn put<'a, P: AsRef<Path>>(path: P, v: Vec<u8>) {
        FILES.lock().unwrap().insert(path.as_ref().into(), Arc::new(Mutex::new(Cursor::new(v))));
    }
}
