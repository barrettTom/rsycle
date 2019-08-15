extern crate clap;
extern crate dirs;

use clap::{App, AppSettings::ArgRequiredElseHelp, Arg};
use csv::{ReaderBuilder, Writer};
use std::fs;
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

    if matches.is_present("restore") {
        if let Some(filename) = matches.value_of("INPUT") {
            let filename = filename.to_owned() + ".";
            restore(rsyclebin, &filename);
        } else {
            restore_cli(rsyclebin);
        }
    } else if matches.is_present("empty") {
        empty(rsyclebin);
    } else if matches.is_present("list") {
        list(rsyclebin);
    } else if let Some(filename) = matches.value_of("INPUT") {
        rsycle(rsyclebin, filename);
    }
}

fn log(rsyclebin: PathBuf, old_path: PathBuf, new_path: PathBuf) {
    let mut rsyclebin_log = rsyclebin.clone();
    rsyclebin_log.push(".log");

    let file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(rsyclebin_log)
        .unwrap();
    let mut writer = Writer::from_writer(file);

    writer
        .write_record(&[
            fs::canonicalize(old_path).unwrap().to_str().unwrap(),
            new_path.to_str().unwrap(),
        ])
        .unwrap();
    writer.flush().unwrap();
}

fn rsycle(rsyclebin: PathBuf, filename: &str) {
    let old_path: PathBuf = [Component::CurDir, Component::Normal(filename.as_ref())]
        .iter()
        .collect();

    if !old_path.exists() {
        println!("File not found!");
        return;
    }

    let new_filename = filename.to_owned()
        + "."
        + &SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

    let mut new_path = rsyclebin.clone();
    new_path.push(new_filename);

    log(rsyclebin, old_path.clone(), new_path.clone());
    fs::rename(old_path, new_path).unwrap();
}

fn restore(rsyclebin: PathBuf, filename: &str) {
    let paths: Vec<PathBuf> = fs::read_dir(rsyclebin.clone())
        .unwrap()
        .map(Result::unwrap)
        .map(|dir| dir.path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(filename)
        })
        .collect();

    match paths.iter().max_by_key(|path| {
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
    }) {
        Some(latest_path) => {
            let original_path = find_original_path(rsyclebin, latest_path.to_path_buf());
            fs::rename(latest_path, original_path).unwrap();
        }
        None => println!("No file found !"),
    }
}

fn list(rsyclebin: PathBuf) {
    let mut paths: Vec<PathBuf> = fs::read_dir(rsyclebin.clone())
        .unwrap()
        .map(Result::unwrap)
        .map(|dir| dir.path())
        .collect();

    for current_path in paths.iter_mut() {
        let mut filename: Vec<&str> = current_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .collect();

        if let Some(last_point) = filename.pop() {
            if let Ok(date) = last_point.parse::<u64>() {
                let original_path = find_original_path(rsyclebin.clone(), current_path.clone());
                let date = UNIX_EPOCH + Duration::from_secs(date);
                println!("{:?}, {:?}, {:?}", original_path, current_path, date);
            }
        }
    }
}

fn find_original_path(rsyclebin: PathBuf, path: PathBuf) -> String {
    let mut rsyclebin_log = rsyclebin.clone();
    rsyclebin_log.push(".log");

    let file = fs::OpenOptions::new()
        .read(true)
        .open(rsyclebin_log)
        .unwrap();

    let mut reader = ReaderBuilder::new().has_headers(false).from_reader(file);

    let path_str = path.to_str().unwrap();

    reader
        .records()
        .map(Result::unwrap)
        .find(|line| line.get(1).unwrap() == path_str)
        .map(|line| line.get(0).unwrap().to_string())
        .unwrap()
}

fn empty(rsyclebin: PathBuf) {
    fs::remove_dir_all(rsyclebin).unwrap()
}

fn restore_cli(_rsyclebin: PathBuf) {
    println!("unimplemented!")
}
