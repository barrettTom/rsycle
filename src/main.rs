extern crate clap;
extern crate dirs;

use clap::{App, AppSettings::ArgRequiredElseHelp, Arg};
use csv::{ReaderBuilder, Writer};
use std::fs;
use std::io;
use std::path::{Component, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    let matches = App::new("rsycle")
        .author("Tom B. <tom@tombarrett.xyz>")
        .version("0.1.0")
        .about("Rsycle your files via cli")
        .arg(
            Arg::with_name("restore")
                .long("restore")
                .help("Restore INPUT file if given, if not open restore prompt"),
        )
        .arg(
            Arg::with_name("empty")
                .long("empty")
                .help("Empties the rsyclebin"),
        )
        .arg(
            Arg::with_name("list")
                .long("list")
                .help("List all files located in the rsyclebin"),
        )
        .arg(Arg::with_name("INPUT").help("File that will be recycled / restored"))
        .setting(ArgRequiredElseHelp)
        .get_matches();

    let mut rsyclebin = dirs::home_dir().unwrap();
    rsyclebin.push(".rsyclebin");
    if !rsyclebin.exists() {
        fs::create_dir(&rsyclebin).unwrap();
    }

    if let Some(filename) = matches.value_of("INPUT") {
        let path = build_path(filename).unwrap();
        if matches.is_present("restore") {
            restore(rsyclebin, path).unwrap();
        } else {
            rsycle(rsyclebin, path).unwrap();
        }
    } else if matches.is_present("empty") {
        empty(rsyclebin).unwrap();
    } else if matches.is_present("list") {
        list(rsyclebin).unwrap();
    } else if matches.is_present("restore") {
        restore_cli(rsyclebin);
    }
}

pub fn build_path(filename: &str) -> Result<PathBuf, io::Error> {
    let relative: PathBuf = [Component::CurDir, Component::Normal(filename.as_ref())]
        .iter()
        .collect();

    if relative.exists() {
        fs::canonicalize(&relative)
    } else {
        fs::File::create(relative.clone())?;
        let full_path = fs::canonicalize(&relative.clone())?;
        fs::remove_file(relative)?;
        Ok(full_path)
    }
}

pub fn rsycle(rsyclebin: PathBuf, old_path: PathBuf) -> Result<(), io::Error> {
    if !old_path.exists() {
        Err(io::Error::new(io::ErrorKind::NotFound, "File not found!"))
    } else {
        let new_filename = old_path.file_name().unwrap().to_str().unwrap().to_owned()
            + "."
            + &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string();

        let mut new_path = rsyclebin.clone();
        new_path.push(new_filename);

        log(rsyclebin, old_path.clone(), new_path.clone())?;
        fs::rename(old_path, new_path)
    }
}

fn log(rsyclebin: PathBuf, old_path: PathBuf, new_path: PathBuf) -> Result<(), io::Error> {
    let mut rsyclebin_log = rsyclebin.clone();
    rsyclebin_log.push(".log");

    let file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(rsyclebin_log)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&[
        fs::canonicalize(old_path)?.to_str().unwrap(),
        new_path.to_str().unwrap(),
    ])?;
    writer.flush()
}

pub fn restore(rsyclebin: PathBuf, original_path: PathBuf) -> Result<(), io::Error> {
    let current_path = most_recent_current_path(rsyclebin, original_path.clone())?;

    if !original_path.exists() {
        fs::rename(current_path, original_path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "There is a file in the way!",
        ))
    }
}

pub fn most_recent_current_path(rsyclebin: PathBuf, path: PathBuf) -> Result<PathBuf, io::Error> {
    let mut rsyclebin_log = rsyclebin.clone();
    rsyclebin_log.push(".log");

    let file = fs::OpenOptions::new().read(true).open(rsyclebin_log)?;

    let mut reader = ReaderBuilder::new().has_headers(false).from_reader(file);

    let path_str = path.to_str().unwrap();

    let original_paths: Vec<PathBuf> = reader
        .records()
        .map(Result::unwrap)
        .filter(|line| line.get(0).unwrap() == path_str)
        .map(|line| PathBuf::from(line.get(1).unwrap()))
        .collect();

    Ok(original_paths
        .iter()
        .max_by_key(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split('.')
                .collect::<Vec<&str>>()
                .pop()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        })
        .unwrap()
        .to_path_buf())
}

pub fn list(rsyclebin: PathBuf) -> Result<(), io::Error> {
    for current_path in all_paths(rsyclebin.clone())? {
        let mut filename: Vec<&str> = current_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .collect();

        if let Some(last_point) = filename.pop() {
            if let Ok(date) = last_point.parse::<u64>() {
                let original_path = find_original_path(rsyclebin.clone(), current_path.clone())?;
                let date = UNIX_EPOCH + Duration::from_secs(date);
                println!(
                    "original: {:?}, current: {:?}, date: {:?}",
                    original_path, current_path, date
                );
            }
        }
    }

    Ok(())
}

fn find_original_path(rsyclebin: PathBuf, current_path: PathBuf) -> Result<PathBuf, io::Error> {
    let mut rsyclebin_log = rsyclebin.clone();
    rsyclebin_log.push(".log");

    let file = fs::OpenOptions::new().read(true).open(rsyclebin_log)?;

    let mut reader = ReaderBuilder::new().has_headers(false).from_reader(file);

    let path_str = current_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cannot convert."))?;

    Ok(PathBuf::from(
        reader
            .records()
            .map(Result::unwrap)
            .find(|line| line.get(1).unwrap() == path_str)
            .map(|line| line.get(0).unwrap().to_string())
            .unwrap(),
    ))
}

pub fn empty(rsyclebin: PathBuf) -> Result<(), io::Error> {
    for path in all_paths(rsyclebin)? {
        if fs::remove_file(path.clone()).is_err() {
            fs::remove_dir_all(path)?
        }
    }

    Ok(())
}

fn all_paths(rsyclebin: PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir(rsyclebin.clone())?
        .map(Result::unwrap)
        .map(|dir| dir.path())
        .collect())
}

fn restore_cli(_rsyclebin: PathBuf) {
    println!("unimplemented!")
}
