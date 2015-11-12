use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::{PathBuf, Path};

pub trait Filesystem {
  type FSRead: Read;
  type FSWrite: Write;
  fn make_absolute<P: AsRef<Path>>(p: P) -> io::Result<PathBuf>;
  fn open<P: AsRef<Path>>(p: P) -> io::Result<Self::FSRead>;
  fn can_open<P: AsRef<Path>>(p: P) -> io::Result<()>;
  fn save<P: AsRef<Path>>(p: P) -> io::Result<Self::FSWrite>;
  fn can_save<P: AsRef<Path>>(p: P) -> io::Result<()>;
}

pub struct DefaultFilesystem;
impl Filesystem for DefaultFilesystem {
    type FSRead = File;
    type FSWrite = File;

    fn make_absolute<P: AsRef<Path>>(p: P) -> io::Result<PathBuf> {
        let mut path = try!(env::current_dir());
        path.push(p);
        Ok(path)
    }

    fn open<P: AsRef<Path>>(p: P) -> io::Result<Self::FSRead> {
        File::open(p)
    }

    fn can_open<P: AsRef<Path>>(p: P) -> io::Result<()> {
        let path = try!(Self::make_absolute(p));

        {
            let parent = match path.parent() {
                Some(path) => path,
                None => return Err(io::Error::new(io::ErrorKind::Other, "Invalid path")),
            };

            let res = try!(fs::metadata(parent));
            // TODO: Add actual testing of permissions, etc...
        }

        let res = try!(fs::metadata(path));
        // TODO: Add actual testing of permissions, etc...

        Ok(())
    }

    fn save<P: AsRef<Path>>(p: P) -> io::Result<Self::FSWrite> {
        File::create(p)
    }

    fn can_save<P: AsRef<Path>>(p: P) -> io::Result<()> {
        let path = try!(Self::make_absolute(p));

        {
            let parent = match path.parent() {
                Some(path) => path,
                None => return Err(io::Error::new(io::ErrorKind::Other, "Invalid path")),
            };

            let res = try!(fs::metadata(parent));
            // TODO: Add actual testing of permissions, etc...
        }

        let res = fs::metadata(path);
        match res {
            Ok(_) => Err(io::Error::new(io::ErrorKind::AlreadyExists, "Already exists")),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            e @ _ => e.map(|_| ()),
        }
    }
}
