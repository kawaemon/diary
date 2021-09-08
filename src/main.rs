use chrono::prelude::*;
use chrono::Duration;

use std::env::var as envvar;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const DEFAULT_EDITOR: &str = "vim";

fn get_default_diary_path() -> PathBuf {
    let mut home = dirs::home_dir().expect("i couldn't find your home directory");
    home.push("diary");
    home
}

fn put_template(file_path: impl AsRef<Path>, date: Date<Local>) -> Result<(), io::Error> {
    let file_path = file_path.as_ref();

    // TODO: i want to put more precious template
    let template = format!("## {}/{}/{}", date.year(), date.month(), date.day());

    let mut file = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(file_path)?;

    let reader = io::BufReader::new(&file);

    // check whether template is already written
    let mut is_already_written = false;
    let mut no_lines = true;
    for line in reader.lines() {
        let line = line?;
        no_lines = false;
        if line == template {
            is_already_written = true;
            break;
        }
    }

    if !is_already_written {
        writeln!(file, "\n\n{}", template)?;
    }

    Ok(())
}

fn main() {
    let diary_dir = envvar("DIARY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| get_default_diary_path());

    if !diary_dir.exists() {
        fs::create_dir_all(&diary_dir).expect("failed to create diary directory");
    }

    let now = Local::now();

    // if it is before 15 o'clock, assume you sat up all night and open yesterdays's diary.
    // i often do btw
    // TODO: should be toggleable by arguments.
    let date = if now.hour() <= 14 {
        now.date() - Duration::days(1)
    } else {
        now.date()
    };

    let mut diary_path = diary_dir.clone();

    let filename = format!("{}{:02}.md", date.year(), date.month());
    diary_path.push(filename);

    put_template(&diary_path, date).expect("failed to write template");

    let editor = envvar("DIARY_EDITOR")
        .or(envvar("EDITOR"))
        .unwrap_or_else(|_| DEFAULT_EDITOR.into());

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&format!("{} {}", editor, diary_path.display()))
        .spawn()
        .expect("failed to launch editor")
        .wait()
        .expect("couldn't wait for closing editor");

    if !status.success() {
        println!("editor exited with non-ok status code");
        return;
    }

    let mut diary_git_path = diary_dir.clone();
    diary_git_path.push(".git");

    if !diary_git_path.exists() {
        return;
    }

    let status = std::process::Command::new("git")
        .current_dir(&diary_dir)
        .arg("add")
        .arg(&diary_path)
        .status()
        .expect("failed to run `git add {diary_path}`");

    assert!(status.success(), "failed to run `git add {diary_path}`");

    let status = std::process::Command::new("git")
        .current_dir(&diary_dir)
        .arg("commit")
        .arg("-m")
        .arg(format!(
            "auto commit for {}/{}/{} diary",
            date.year(),
            date.month(),
            date.day()
        ))
        .status()
        .expect("failed to commit");

    assert!(status.success(), "failed to commit");

    println!("automatically committed");
}
