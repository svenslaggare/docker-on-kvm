use std::path::Path;
use regex::Regex;

#[derive(Debug)]
pub struct LinuxKernel {
    pub kernel: String,
    pub initrd: String,
    pub version: String,
    pub active: bool
}

impl LinuxKernel {
    pub fn find() -> std::io::Result<Vec<LinuxKernel>> {
        let boot_path = Path::new("/boot");
        let dir_entries = std::fs::read_dir(boot_path)?;

        let active_kernel = boot_path.join("vmlinuz").canonicalize().ok();

        let mut kernels = Vec::new();
        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap().to_str().unwrap().to_owned();
                if let Some(kernel_match) = Regex::new("vmlinuz-(.*)").unwrap().captures(&filename) {
                    let version = kernel_match.get(1).unwrap().as_str();
                    let initrd_path = boot_path.join(format!("initrd.img-{}", version));

                    if initrd_path.exists() {
                        kernels.push(LinuxKernel {
                            kernel: path.to_str().unwrap().to_owned(),
                            initrd: initrd_path.to_str().unwrap().to_owned(),
                            version: version.to_owned(),
                            active: active_kernel.as_ref().map(|kernel| kernel == &path).unwrap_or(false)
                        });
                    }
                }
            }
        }

        kernels.sort_by_key(|kernel| kernel.version.clone());
        Ok(kernels)
    }
}