use std::path::{Path, PathBuf};

use crate::helpers::run_command;
use crate::helpers;

#[derive(Debug)]
pub enum DockerImageError {
    IO(std::io::Error),
    ImageNotFound,
    DestinationExist,
    FailedToExtract(String),
    FailedToBuild
}

pub fn try_extract_image(image_id: &str, destination_folder: &Path) -> Result<PathBuf, DockerImageError> {
    let image_hash = run_command("docker", &["inspect", image_id, "--format={{ .Id }}"])
        .map_err(|_| DockerImageError::ImageNotFound)?;
    let image_hash = image_hash.trim().split("sha256:").skip(1).next().unwrap().to_owned();

    let destination = destination_folder.join(image_hash);

    match extract_image_filesystem(image_id, &destination) {
        Ok(_) => Ok(destination),
        Err(err) => {
            match err {
                DockerImageError::DestinationExist => Ok(destination),
                err => Err(err)
            }
        }
    }
}

pub fn extract_image_filesystem(image_id: &str, destination: &Path) -> Result<(), DockerImageError> {
    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            std::fs::create_dir(parent).map_err(|err| DockerImageError::IO(err))?;
        }
    }

    if destination.exists() {
        return Err(DockerImageError::DestinationExist);
    }

    std::fs::create_dir(destination).map_err(|err| DockerImageError::IO(err))?;

    let container_id = run_command("docker", &["create", image_id])
        .map_err(|_| DockerImageError::ImageNotFound)?;
    let container_id = container_id.trim();

    let tmp_export_path = helpers::temp_filename(".tar");
    let tmp_export_path_str = tmp_export_path.to_str().unwrap();

    let destination_str = destination.to_str().unwrap();

    let mut results = Vec::new();
    results.push(run_command("docker", &["export", &container_id, "--output", tmp_export_path_str])
        .map(|_| ())
        .map_err(|err| DockerImageError::FailedToExtract(err))
    );

    match results.last() {
        Some(last) if last.is_ok() => {
            results.push(run_command("sudo", &["-S", "tar", "--same-owner", "-xvf", tmp_export_path_str, "--directory", destination_str])
                .map(|_| ())
                .map_err(|err| DockerImageError::FailedToExtract(err))
            );

            // Docker messes with /etc/resolv.conf. We re-creates the symlink with what systemd-resolved updates
            results.push(run_command("sudo", &["-S", "bash", "-c", &format!("rm -f {root_dir}/etc/resolv.conf ; ln -s /run/systemd/resolve/resolv.conf {root_dir}/etc/resolv.conf", root_dir = destination_str)])
                .map(|_| ())
                .map_err(|err| DockerImageError::FailedToExtract(err))
            );
        }
        _ => {}
    }

    results.push(run_command("docker", &["rm", &container_id])
        .map(|_| ())
        .map_err(|err| DockerImageError::FailedToExtract(err))
    );

    results.push(std::fs::remove_file(tmp_export_path)
        .map_err(|err| DockerImageError::IO(err))
    );

    for result in results {
        if let Err(err) = result {
            std::fs::remove_dir_all(destination).map_err(|err| DockerImageError::IO(err))?;
            return Err(err);
        }
    }

    Ok(())
}

pub fn build(filename: &Path, tag: &str) -> Result<(), DockerImageError> {
    let mut command = std::process::Command::new("docker");
    command
        .env("LANG", "en")
        .args(&["build", "-t", tag, "-f", filename.to_str().unwrap(), "."]);

    let mut child = command.spawn().map_err(|_| DockerImageError::FailedToBuild)?;
    let status = child.wait().map_err(|_| DockerImageError::FailedToBuild)?;

    if status.success() {
        Ok(())
    } else {
        Err(DockerImageError::FailedToBuild)
    }
}