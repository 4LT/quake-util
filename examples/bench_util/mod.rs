use std::fs::{create_dir_all, read_to_string, File};
use std::process::Command;

const LARGE_FILE_PATH: &str = "test-res/lg-file";
const LARGE_FILE_HASH_PATH: &str = "test-res/lg-file-hash";
const HASH_SUFFIX: &str = "sha256";
const DOWNLOAD_PFX: &str =
    "https://quake-util-test-res.nyc3.digitaloceanspaces.com";
const HASHER: &str = "sha256sum";

pub fn strip_hash<E>(text_res: Result<&str, E>) -> Result<String, String>
where
    E: std::fmt::Display,
{
    text_res
        .map(|line| line.split_ascii_whitespace().next().map(String::from))
        .map_err(|e| format!("{e}"))?
        .ok_or(String::from("Missing hash"))
}

pub fn hash(path: &str) -> Result<String, String> {
    println!("Hashing file {path} ...");

    Command::new(HASHER)
        .arg(path)
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap())
        .map_err(|e| format!("Failed to hash file: {e}"))
}

pub fn download_file(filename: &str, out_path: &str) -> Result<(), String> {
    let url = format!("{DOWNLOAD_PFX}/{filename}");

    println!("Downloading {url} to {out_path} ...");

    let status_res = Command::new("curl").args(["-o", out_path, &url]).status();

    match status_res {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!("Failed to download {url}"))
            }
        }
        Err(e) => Err(format!("Failed to download file: {e}")),
    }
}

pub fn prepare_file(filename: &str) -> Result<String, String> {
    let hash_path = format!("{LARGE_FILE_HASH_PATH}/{filename}.{HASH_SUFFIX}");
    let file_path = format!("{LARGE_FILE_PATH}/{filename}");
    let expected_hash = strip_hash(read_to_string(hash_path).as_deref())?;
    let file_res = File::open(&file_path);
    let _ = create_dir_all(LARGE_FILE_PATH);

    let mut did_download = match file_res {
        Ok(_) => false,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                download_file(filename, &file_path)?;
            } else {
                return Err(format!("{e}"));
            }
            true
        }
    };

    let mut attempts = 0;
    let mut hashes_match = false;

    let required_attempts = if did_download { 2 } else { 1 };

    while !hashes_match && attempts < required_attempts {
        let actual_hash = strip_hash(hash(&file_path).as_deref())?;
        hashes_match = actual_hash == expected_hash;
        attempts += 1;

        if hashes_match {
            println!("Hashes match");
        } else {
            println!("Hashes do not match");
        }

        if !hashes_match && !did_download {
            download_file(filename, &file_path)?;
            did_download = true;
        }
    }

    if hashes_match {
        Ok(file_path)
    } else {
        Err(String::from("Stopping..."))
    }
}
