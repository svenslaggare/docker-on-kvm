use std::path::Path;

use regex::Regex;

use crate::definition::FileSystem;
use crate::helpers::run_command;

#[derive(Debug)]
pub enum DiskFormat {
    Raw,
    Qcow2
}

impl std::fmt::Display for DiskFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskFormat::Raw => write!(f, "raw"),
            DiskFormat::Qcow2 => write!(f, "qcow2")
        }
    }
}

#[derive(Debug)]
pub enum DiskCreateError {
    DiskAlreadyExists,
    DirectoryNotExist,
    BackingFileNotExist,
    FailedCreate
}

pub fn create_from_directory(disk_file: &Path,
                             disk_size_in_bytes: u64,
                             file_system: FileSystem,
                             directory: &Path) -> Result<(), DiskCreateError> {
    if disk_file.exists() {
        return Err(DiskCreateError::DiskAlreadyExists);
    }

    if !directory.exists() {
        return Err(DiskCreateError::DirectoryNotExist);
    }

    let tmp_mount_path = Path::new("/tmp/docker-on-kvm-mount");
    if !tmp_mount_path.exists() {
        std::fs::create_dir(tmp_mount_path).unwrap();
    }

    let tmp_mount_path_str = tmp_mount_path.to_str().unwrap();
    let disk_file_str = disk_file.to_str().unwrap();

    run_command("fallocate", &["-l", &disk_size_in_bytes.to_string(), disk_file_str])
        .map_err(|_| DiskCreateError::FailedCreate)?;

    match file_system {
        FileSystem::Ext4 => {
            run_command("mkfs.ext4", &["-F", disk_file_str])
                .map_err(|_| DiskCreateError::FailedCreate)?;

            run_command("sudo", &["-S", "mount", "-t", "ext4", "-o", "loop", disk_file_str, tmp_mount_path_str])
                .map_err(|_| DiskCreateError::FailedCreate)?;
        }
    }

    let result = run_command(
        "sudo",
        &["-S", "cp", "-ax", directory.join(".").to_str().unwrap(), tmp_mount_path.join(".").to_str().unwrap()]
    ).map_err(|err| {
        println!("{}", err);
        DiskCreateError::DirectoryNotExist
    });

    run_command("sudo", &["-S", "umount", tmp_mount_path_str])
        .map_err(|_| DiskCreateError::FailedCreate)?;

    result?;
    Ok(())
}

pub fn create_copy_on_write_image(disk_file: &Path, backing_file: &Path) -> Result<(), DiskCreateError> {
    if disk_file.exists() {
        return Err(DiskCreateError::DiskAlreadyExists);
    }

    if !backing_file.exists() {
        return Err(DiskCreateError::BackingFileNotExist);
    }

    run_command(
        "qemu-img",
        &["create", "-f", "qcow2", "-o", &format!("backing_file={}", backing_file.canonicalize().unwrap().to_str().unwrap()), disk_file.to_str().unwrap()]
    ).map_err(|_| DiskCreateError::FailedCreate)?;

    Ok(())
}

#[derive(Debug)]
pub struct DiskInfo {
    pub format: DiskFormat,
    pub backing_file: Option<String>
}

impl DiskInfo {
    pub fn for_disk_file(disk: &Path) -> Option<DiskInfo> {
        let output = run_command("qemu-img", &["info", disk.to_str().unwrap()]).ok()?;

        let mut format = None;
        let mut backing_file = None;

        for line in output.lines() {
            if let Some(regex_match) = Regex::new("file format: (.*)").unwrap().captures(line) {
                match regex_match.get(1).unwrap().as_str() {
                    "raw" => { format = Some(DiskFormat::Raw); },
                    "qcow2" => { format = Some(DiskFormat::Qcow2); }
                    _ => {}
                }
            } else if let Some(regex_match) = Regex::new("backing file: (.*)").unwrap().captures(line) {
                backing_file = Some(regex_match.get(1).unwrap().as_str().to_owned());
            }
        }

        Some(DiskInfo {
            format: format?,
            backing_file
        })
    }
}