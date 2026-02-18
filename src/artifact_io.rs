use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use crate::default_config::DEFAULT_CONFIG_TOML;

pub fn read_text_file(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut text = String::new();
    reader.read_to_string(&mut text)?;
    Ok(text)
}

pub fn write_text_file(path: &Path, text: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(text.as_bytes())?;
    writer.flush()
}

pub fn write_text_file_if_missing(path: &Path, text: &str) -> io::Result<bool> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => {
            let mut writer = BufWriter::new(file);
            writer.write_all(text.as_bytes())?;
            writer.flush()?;
            Ok(true)
        }
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(err) => Err(err),
    }
}

pub fn home_dir() -> io::Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))
}

pub fn metaagent_config_file_path() -> io::Result<PathBuf> {
    let config_dir = home_dir()?.join(".metaagent");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.toml"))
}

pub fn ensure_default_metaagent_config() -> io::Result<PathBuf> {
    let config_file = metaagent_config_file_path()?;
    write_text_file_if_missing(&config_file, DEFAULT_CONFIG_TOML)?;
    Ok(config_file)
}
