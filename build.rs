use std::{
    env,
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use chrono;

fn get_git_version() -> Result<String, Box<dyn Error>> {
    use std::process::Command;
    use std::str;

    let branch_name = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()?;
    let branch_name = str::from_utf8(&branch_name.stdout)?
        .trim_end();

    let latest_tag = Command::new("git")
        .arg("describe")
        .arg("--tags")
        .arg("--abbrev=0")
        .output()?;
    let latest_tag = str::from_utf8(&latest_tag.stdout)?
        .trim_end();

    let commit_short = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()?;
    let commit_short = str::from_utf8(&commit_short.stdout)?
        .trim_end();

    let tag_short = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg(&latest_tag)
        .output()?;
    let tag_short = str::from_utf8(&tag_short.stdout)?
        .trim_end();

    if tag_short == commit_short {
        Ok(String::new()) // Favor version at Cargo.toml
    } else {
        Ok(String::from(format!("{}-{}", commit_short, branch_name)))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let outdir   = env::var("OUT_DIR")?;

    let timestamp = chrono::offset::Local::now();
    let timestamp = timestamp.format("%Y-%m-%d %H:%M:%S");
    let destpath  = Path::new(&outdir).join("timestamp.txt");
    let mut file  = BufWriter::new(File::create(&destpath)?);
    write!(file, "{}", timestamp)?;

    let version  = get_git_version()?;
    let destpath = Path::new(&outdir).join("version.txt");
    let mut file = BufWriter::new(File::create(&destpath)?);
    write!(file, "{}", version)?;

    let target   = env::var("TARGET")?;
    let destpath = Path::new(&outdir).join("target.txt");
    let mut file = BufWriter::new(File::create(&destpath)?);
    write!(file, "{}", target)?;

    Ok(())
}
