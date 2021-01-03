use std::ffi::OsStr;
use std::process::Stdio;
use std::path::{PathBuf, Path};

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn run_command<I, S>(command: &str, args: I) -> Result<String, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr> {
    let mut command = std::process::Command::new(command);
    command
        .env("LANG", "en")
        .stdin(Stdio::inherit())
        .args(args);

    match command.output() {
        Ok(result) => {
            if result.status.success() {
                Ok(String::from_utf8(result.stdout).unwrap())
            } else {
                Err(String::from_utf8(result.stderr).unwrap())
            }
        }
        Err(_) => Err(String::new())
    }
}

pub fn temp_filename(suffix: &str) -> PathBuf {
    let rand_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    std::env::temp_dir().join(Path::new(&format!("{}{}", rand_name, suffix)))
}