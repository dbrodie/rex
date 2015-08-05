
use std::default::Default;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io;
use std::env;
use std::io::Read;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;
use toml;

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
            ConfigError::InvalidFieldName(ref s) => "Invalid field name",
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
pub struct Config {
    pub show_ascii: bool,
    pub show_linenum: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            show_ascii: true,
            show_linenum: true,
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
    ($obj:expr, $name:ident, $table:expr, $toml_type:ident) => ({
        $obj.$name = match $table.remove(stringify!($name)) {
            Some(val) => try_unwrap_toml!(val, $toml_type),
            None => $obj.$name
        };
    })
}

impl Config {
    fn apply_toml(&mut self, mut t: toml::Table) -> Result<(), ConfigError> {
        decode_toml!(self, show_ascii, t, Boolean);
        decode_toml!(self, show_linenum, t, Boolean);
        Ok(())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        let mut s = String::new();
        let mut f = try!(File::open(path));
        try!(f.read_to_string(&mut s));
        let mut parser =  toml::Parser::new(&s);
        if let Some(table) = parser.parse() {
            let mut config: Config = Default::default();
            config.apply_toml(table);
            return Ok(config);
        }
        Err(ConfigError::TomlParserErrors(parser.errors))
    }

    fn get_config_path() -> PathBuf {
        let mut p = PathBuf::new();
        p.push(env::var("XDG_CONFIG_HOME").unwrap_or_else(
                |e| env::var("HOME").unwrap_or("/".into()) + "/.config"
            ));
        p.join("hyksa").join("hyksa.conf")
    }

    pub fn open_default() -> Result<Config, ConfigError> {
        Config::from_file(Config::get_config_path())
    }
}
