
use std::default::Default;
use std::path::{Path};
use std::io;
use std::io::{Read, Write};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;
use std::marker::PhantomData;
use toml;

use super::filesystem::Filesystem;

pub use toml::Value;

/// An error reading, writing, or parsing configuration data
#[derive(Debug)]
pub enum ConfigError {
    /// An IO error occured during reading/writing the config data
    IoError(io::Error),
    /// A basic parsing error
    TomlParserErrors(Vec<toml::ParserError>),
    /// The field name supplied is invalid
    InvalidFieldName(String),
    /// The type of value for the field name does not match
    InvalidFieldType(& 'static str, String),
    /// The value provided for the field name is invalid
    InvalidFieldValue(& 'static str),
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::IoError(ref e) => e.description(),
            ConfigError::TomlParserErrors(ref v) => v[0].description(),
            ConfigError::InvalidFieldName(_) => "Invalid field name",
            ConfigError::InvalidFieldType(_, _) => "Invalid field type",
            ConfigError::InvalidFieldValue(_) => "Invalid field value",
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
            ConfigError::InvalidFieldName(ref s) => write!(f, "Invalid field name: {}", s),
            ConfigError::InvalidFieldType(expected, ref got) => write!(f, "Expected type {} got {}", expected, got),
            ConfigError::InvalidFieldValue(field_name) => write!(f, "Invalid field value: {}", field_name),
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
    pub group_bytes: i64,
    pub little_endian: bool,

    _fs: PhantomData<FS>
}

impl<FS: Filesystem+'static> Default for Config<FS> {
    fn default() -> Config<FS> {
        Config {
            show_ascii: true,
            show_linenum: true,
            line_width: None,
            group_bytes: 1,
            little_endian: false,
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

/// Macro simplifying the decoding of toml values to the config. Can be used in two forms:
/// ```decode_toml!(config, field_name, toml_value)``` or
/// ```decode_toml!(config, field_name, toml_value, error_value, |value| Option<mapped_value>)```.
/// ```config``` - The config object, probably should be ```self```.
/// ```field_name``` - The field name in the config struct and the toml table.
/// ```toml_type``` - The ```toml::Value``` type that this field is mapped to. By default, this means the field
///     in the struct should be of the same type as ```toml::Value::$toml_value```. If something more
///     complicated is needed, use a map function.
/// [map_function] - Converts a value from the ```toml::Value``` to a Result<T> where T is the
///     type in the struct.
macro_rules! decode_toml {
    ($obj:expr, $name:ident, $table:expr, $toml_type:ident, $map_filter_func:expr) => ({
        $obj.$name = match $table.remove(stringify!($name)) {
            Some(val) => try!($map_filter_func(try_unwrap_toml!(val, $toml_type))),
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

/// Macro simplifying the creation of toml values from the config. Can be used in two forms:
/// ```create_toml!(config, position, field_name, toml_value)``` or
/// ```create_toml!(config, position, field_name, toml_value, |value| <mapped_value>)```.
/// ```config``` - The config object, probably should be ```self```.
/// ```position``` - The numeric mapping from a usize to the field type. This allows to create each line
///     based on an index of the configuration option. This value is modified for each macro use.
/// ```field_name``` - The field name in the config struct and the toml table.
/// ```toml_type``` - The ```toml::Value``` type that this field is mapped to. By default, this means the field
///     in the struct should be of the same type as ```toml::Value::$toml_value```. If something more
///     complicated is needed, use a map function.
/// ```[map_function]``` - Converts a value from the type in the config struct to the proper ```toml::Value```.
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
    fn apply_toml(&mut self, mut t: toml::Table) -> Result<(), ConfigError> {
        decode_toml!(self, show_ascii, t, Boolean);
        decode_toml!(self, show_linenum, t, Boolean);
        decode_toml!(self, line_width, t, Integer, |i|
            if i > 0 {
                Ok(Some(i as u32))
            } else if i == 0 {
                Ok(None)
            } else {
                Err(ConfigError::InvalidFieldValue("line_width must be >= 0"))
            }
        );
        decode_toml!(self, group_bytes, t, Integer, |i|
            if i <= 64 && i >= 0 {
                Ok(i)
            } else {
                Err(ConfigError::InvalidFieldValue("group_bytes must be between 0 and 64"))
            }
        );
        decode_toml!(self, little_endian, t, Boolean);
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
        create_toml!(self, p, group_bytes, Integer);
        create_toml!(self, p, little_endian, Boolean);
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

    pub fn from_file<P: AsRef<Path>>(path: Option<P>) -> Result<Config<FS>, ConfigError> {
        let mut config: Config<FS> = Default::default();
        if let Some(p) = path {
            let mut s = String::new();
            let mut f = try!(FS::open(p));
            try!(f.read_to_string(&mut s));
            try!(config.set_from_string(&s));
        }
        Ok(config)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let mut f = try!(FS::save(path));
        for (key, value) in self.values() {
            try!(writeln!(&mut f, "{}={}", key, value));
        }
        Ok(())
    }
    pub fn open_default() -> Result<Config<FS>, ConfigError> {
        Self::from_file(FS::open_config("rex", "rex.conf"))
    }

    pub fn save_default(&self) ->Result<(), ConfigError> {
        self.to_file(try!(FS::save_config("rex", "rex.conf")))
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
