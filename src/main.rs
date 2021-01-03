use std::path::Path;

use structopt::StructOpt;

use virt::connect::Connect;
use virt::domain::Domain;

mod helpers;
mod definition;
mod disk_creator;
mod docker_image;
mod kernel;

use crate::definition::{VirtualMachine, Disk, FileSystem};
use crate::kernel::LinuxKernel;

#[derive(Debug, StructOpt)]
#[structopt(name="docker-on-kvm", about="Run docker images as KVM VMs")]
enum CommandLineInput {
    #[structopt(about="Runs docker image")]
    Run {
        #[structopt(name="docker_image", help="The tag of the docker image to run")]
        docker_image: String,
        #[structopt(name="name", help="The name of the VM")]
        name: String,
        #[structopt(long, help="The size of the disk in megabytes.", default_value="2048")]
        disk_size: u64,
        #[structopt(long, help="The amount of RAM in megabytes.", default_value="2048")]
        ram_size: u64,
        #[structopt(long, help="The number of CPU cores", default_value="2")]
        num_cpus: u64,
        #[structopt(long, help="The linux kernel to use.")]
        kernel: Option<String>,
    },
    #[structopt(about="Simple wrapper around docker build")]
    Build {
        #[structopt(name="filename", help="The docker file to build")]
        filename: String,
        #[structopt(name="tag", help="The tag to build as")]
        tag: String
    },
    #[structopt(about="Lists the linux kernels available")]
    ListKernels
}

fn main() {
    let command_line_input = CommandLineInput::from_args();

    match command_line_input {
        CommandLineInput::Run { docker_image, name, disk_size, ram_size, num_cpus, kernel } => {
            let kernels = LinuxKernel::find().unwrap();
            let selected_kernel = match kernel {
                Some(kernel) => {
                    kernels
                        .iter()
                        .find(|linux_kernel| &linux_kernel.version == &kernel)
                        .expect("Could not find the specified linux kernel.")
                },
                None => kernels.last().unwrap()
            };

            let vm_name = name;
            let vm_uuid = uuid::Uuid::new_v4().to_simple().to_string();
            let disk_size_in_megabytes = disk_size;
            let ram_in_megabytes = ram_size;

            let extracted_images_dir = Path::new("extracted-images");
            let disks_dir = Path::new("disks");

            let docker_image_extraction = docker_image::try_extract_image(&docker_image, extracted_images_dir).unwrap();
            let destination_disk = disks_dir.join(format!("{}.img", docker_image_extraction.file_name().unwrap().to_str().unwrap()));

            if !destination_disk.exists() {
                disk_creator::create_from_directory(
                    &destination_disk,
                    disk_size_in_megabytes * 1024 * 1024,
                    FileSystem::Ext4,
                    &docker_image_extraction
                ).unwrap();
            }

            println!("Creating VM {} ({}) using docker image {} and kernel {}", vm_name, vm_uuid, docker_image, selected_kernel.version);

            let cow_disk = disks_dir.join(format!("{}.qcow2", vm_uuid));
            disk_creator::create_copy_on_write_image(
                &cow_disk,
                &destination_disk,
            ).unwrap();
            let destination_disk = cow_disk;

            let vm_definition = VirtualMachine {
                name: vm_name,
                kernel_file: selected_kernel.kernel.clone(),
                initrd_file: selected_kernel.initrd.clone(),
                root_disk: Disk::File {
                    filename: destination_disk.canonicalize().unwrap().to_str().unwrap().to_owned(),
                    filesystem: FileSystem::Ext4
                },
                ram_in_bytes: ram_in_megabytes * 1024 * 1024,
                num_cpus
            }.get_xml().unwrap();

            create_and_start_vm(&vm_definition);
        }
        CommandLineInput::Build { filename, tag } => {
            docker_image::build(Path::new(&filename), &tag).unwrap();
        },
        CommandLineInput::ListKernels => {
            println!("Found the following linux kernels:");
            for kernel in LinuxKernel::find().unwrap() {
                println!("{} (path: {}, active: {})", kernel.version, kernel.kernel, kernel.active)
            }
        }
    }
}

fn create_and_start_vm(vm_definition: &str) {
    let uri = "qemu:///system";
    println!("Attempting to connect to hypervisor: '{}'", uri);

    let conn = match Connect::open(&uri) {
        Ok(c) => c,
        Err(e) => panic!(
            "No connection to hypervisor: code {}, message: {}",
            e.code, e.message
        ),
    };

    match conn.get_uri() {
        Ok(u) => println!("Connected to hypervisor at '{}'", u),
        Err(e) => {
            disconnect(conn);
            panic!(
                "Failed to get URI for hypervisor connection: code {}, message: {}",
                e.code, e.message
            );
        }
    };

    let domain = Domain::define_xml(&conn, &vm_definition).unwrap();
    let status = domain.create().unwrap();
    if status == 0 {
        println!("Created VM.");
    } else {
        println!("Failed to create VM: {}", status);
    }

    fn disconnect(mut conn: Connect) {
        if let Err(e) = conn.close() {
            panic!(
                "Failed to disconnect from hypervisor: code {}, message: {}",
                e.code, e.message
            );
        }
        println!("Disconnected from hypervisor");
    }
}