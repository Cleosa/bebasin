use std::env::{current_dir, current_exe};
use std::fs;
use std::io::{self, Read};
use std::io::Write as _;
use std::path::Path;

use serde::Deserialize;

use crate::{CURRENT_VERSION, DEFAULT_HOSTS, HOSTS_HEADER, LATEST_VERSION_URL, UPDATE_URL};
use crate::error::ErrorKind;
use crate::os::{HOSTS_BACKUP_PATH, HOSTS_PATH};
use crate::parser::{parse_from_file, write_to_file};

pub fn is_installed() -> bool {
    // Maybe there are another condition that can be checked
    is_backed()
}

pub fn hosts_exists() -> bool {
    Path::new(HOSTS_PATH).exists()
}

pub fn create_default_hosts() -> io::Result<()> {
    let mut file = fs::File::create(HOSTS_PATH)?;
    file.write_all(format!("{}\n{}", HOSTS_HEADER, DEFAULT_HOSTS).as_bytes())?;
    Ok(())
}

pub fn remove_temp_file() {
    let mut tmp_file = current_dir().unwrap();
    tmp_file.push(".bebasin_tmp");
    if tmp_file.exists() {
        fs::remove_file(tmp_file);
    }
}

pub fn is_backed() -> bool {
    Path::new(HOSTS_BACKUP_PATH).exists()
}

pub fn backup() -> Result<(), ErrorKind> {
    match parse_from_file(HOSTS_PATH) {
        Ok(hosts_local) => match write_to_file(
            HOSTS_BACKUP_PATH,
            &hosts_local,
            include_str!("../misc/header-backup"),
        ) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}
