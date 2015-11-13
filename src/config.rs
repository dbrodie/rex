
use std::default::Default;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::env;
use std::io::Read;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;
use std::marker::PhantomData;
use toml;

use super::filesystem::Filesystem;

pub use toml::Value;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    TomlParserErrors(Vec<toml::ParserError>),
    InvalidFieldName(String),
    InvalidFieldType(& 'static str, String),
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::IoError(ref e) => e.description(),
            ConfigError::TomlParserErrors(ref v) => v[0].description(),
            ConfigError::InvalidFieldName(_) => "Invalid field name",
            ConfigError::InvalidFieldType(_, _) => "Invalid field type",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ConfigError::IoError(ref e) => Some(e),
            ConfigError::TomlParserErrors(ref v) => Some(&v[0]),
            _ => None
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ConfigError::IoError(ref e) => write!(f, "IO Error: {}", e),
            ConfigError::TomlParserErrors(ref v) => write!(f, "Parser Error ({} total): {}",
                    v.len(), v[0]),
            ConfigError::InvalidFieldName(ref s) => write!(f, "Invalid feild name: {}", s),
            ConfigError::InvalidFieldType(expected, ref got) => write!(f, "Expected type {} got {}", expected, got),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> ConfigError {
        ConfigError::IoError(e)
    }
}

#[derive(RustcDecodable, Debug)]
pub struct Config<FS: Filesystem+'static> {
    pub show_ascii: bool,
    pub show_linenum: bool,
    pub line_width: Option<u32>,

    _fs: PhantomData<FS>
}

impl<FS: Filesystem+'static> Default for Config<FS> {
    fn default() -> Config<FS> {
        Config {
            show_ascii: true,
            show_linenum: true,
            line_width: None,
            _fs: PhantomData,
        }
    }
}

macro_rules! try_unwrap_toml {
    ($e:expr, $t:ident) => ({
        match $e {
            toml::Value::$t(v) => v,
            other => {
                return Err(ConfigError::InvalidFieldType(stringify!($t), format!("{}", other)))
            }
        }
    })
}

macro_rules! decode_toml {
    ($obj:expr, $name:ident, $table:expr, $toml_type:ident, $map_func:expr) => ({
        $obj.$name = match $table.remove(stringify!($name)) {
            Some(val) => $map_func(try_unwrap_toml!(val, $toml_type)),
            None => $obj.$name
        };
    });
    ($obj:expr, $name:ident, $table:expr, $toml_type:ident) => ({
        $obj.$name = match $table.remove(stringify!($name)) {
            Some(val) => try_unwrap_toml!(val, $toml_type),
            None => $obj.$name
        };
    });
}

macro_rules! create_toml {
    ($obj:expr, $pos:ident, $name:ident, $toml_type:ident, $map_func:expr) => ({
        if $pos == 0 {
            return Some((stringify!($name), toml::Value::$toml_type($map_func($obj.$name))))
        } else {
            $pos -= 1;
        }
        let _ = $pos;
    });
    ($obj:expr, $pos:ident, $name:ident, $toml_type:ident) => ({
        if $pos == 0 {
            return Some((stringify!($name), toml::Value::$toml_type($obj.$name)))
        } else {
            $pos -= 1;
        }
        let _ = $pos;
    });
}

impl<FS: Filesystem+'static> Config<FS> {
    pub fn get_config_usage() -> &'static str {
        "
The supported configuration options with their default values:
    show_ascii=true        Shows an ascii representation in the right hand side
    show_linenum=true      Shows the byte offset in the left hand side
    line_width=0           The number of bytes per line, 0 meaning auto-wrap

The configuration can be set in the rex.conf file located in the
$XDG_CONFIG_HOME path (usually, ~/.config/rex/rex.conf). Additionally,
properties can be set on the commandline as rex -C show_ascii=false.
"
    }

    fn apply_toml(&mut self, mut t: toml::Table) -> Result<(), ConfigError> {
        decode_toml!(self, show_ascii, t, Boolean);
        decode_toml!(self, show_linenum, t, Boolean);
        decode_toml!(self, line_width, t, Integer, |i| if i > 0 { Some(i as u32) } else { None });
        if let Some((key, _)) = t.into_iter().next() {
            Err(ConfigError::InvalidFieldName(key))
        } else {
            Ok(())
        }
    }

    fn read_toml(&self, mut p: usize) -> Option<(&'static str, toml::Value)> {
        create_toml!(self, p, show_ascii, Boolean);
        create_toml!(self, p, show_linenum, Boolean);
        create_toml!(self, p, line_width, Integer, |opt_i| if let Some(i) = opt_i { i as i64 } else { 0 });
        None
    }

    pub fn values<'a>(&'a self) -> Values<'a, FS> {
        Values {
            config: self,
            pos: 0,
        }
    }

    pub fn set_from_key_value(&mut self, key: &str, val: &str) -> Result<(), ConfigError> {
        // TODO: We can make this more efficient
        self.set_from_string(&format!("{} = {}", key, val))
    }

    pub fn set_from_string(&mut self, set_line: &str) -> Result<(), ConfigError> {
        let mut parser = toml::Parser::new(&set_line);
        if let Some(table) = parser.parse() {
            return self.apply_toml(table);
        }
        Err(ConfigError::TomlParserErrors(parser.errors))
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config<FS>, ConfigError> {
        let mut s = String::new();
        let mut f = try!(File::open(path));
        try!(f.read_to_string(&mut s));
        let mut config: Config<FS> = Default::default();
        try!(config.set_from_string(&s));
        Ok(config)
    }

    fn get_config_path() -> PathBuf {
        let mut p = PathBuf::new();
        p.push(env::var("XDG_CONFIG_HOME").unwrap_or_else(
                |_| env::var("HOME").unwrap_or("/".into()) + "/.config"
            ));
        p.join("rex").join("rex.conf")
    }

    pub fn open_default() -> Config<FS> {
        Self::from_file(Self::get_config_path()).unwrap_or_else(|_| Default::default())
    }
}

pub struct Values<'a, FS: Filesystem+'static> {
    config: &'a Config<FS>,
    pos: usize,
}

impl<'a, FS:Filesystem+'static> Iterator for Values<'a, FS> {
    type Item = (&'static str, toml::Value);

    fn next(&mut self) -> Option<(&'static str, toml::Value)> {
        let pos = self.pos;
        self.pos += 1;
        self.config.read_toml(pos)
    }
}
