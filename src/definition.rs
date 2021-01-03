use std::path::Path;

use crate::disk_creator::{ DiskInfo};

pub enum FileSystem {
    Ext4
}

impl std::fmt::Display for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSystem::Ext4 => write!(f, "ext4")
        }
    }
}

pub enum Disk {
    File { filename: String, filesystem: FileSystem }
}

pub struct VirtualMachine {
    pub name: String,
    pub kernel_file: String,
    pub initrd_file: String,
    pub root_disk: Disk,
    pub ram_in_bytes: u64,
    pub num_cpus: u64
}

impl VirtualMachine {
    pub fn get_xml(&self) -> Option<String> {
        let (root_filesystem_type, root_disk_device_id, root_disk_xml) = match &self.root_disk {
            Disk::File { filename, filesystem } => {
                let disk_info = DiskInfo::for_disk_file(Path::new(filename))?;
                let backing_file = match disk_info.backing_file {
                    Some(backing_file) => {
                        let backing_file_info = DiskInfo::for_disk_file(Path::new(&backing_file))?;

                        format!(
                            r#"
                            <backingStore type="file">
                                <format type="{}"/>
                                <source file="{}"/>
                            </backingStore>"#,
                            backing_file_info.format,
                            backing_file
                        )
                    }
                    None => String::new()
                };

                let device_id = "vda";
                let xml = format!(
                    r#"
                    <disk type="file" device="disk">
                      <driver name="qemu" type="{format}"/>
                      <source file="{filename}"/>
                      <target dev="{device_id}" bus="virtio"/>
                      <address type="pci" domain="0x0000" bus="0x03" slot="0x00" function="0x0"/>
                      {backing_file}
                    </disk>"#,
                    filename = filename,
                    format = disk_info.format.to_string(),
                    device_id = device_id,
                    backing_file = backing_file
                );

                (filesystem, device_id, xml)
            }
        };

        Some(format!(
            r#"
            <domain type="kvm">
              <name>{name}</name>
              <metadata>
                <libosinfo:libosinfo xmlns:libosinfo="http://libosinfo.org/xmlns/libvirt/domain/1.0">
                  <libosinfo:os id="http://ubuntu.com/ubuntu/18.04"/>
                </libosinfo:libosinfo>
              </metadata>
              <memory unit="B">{ram_in_bytes}</memory>
              <currentMemory unit="B">{ram_in_bytes}</currentMemory>
              <vcpu placement="static">{num_cpus}</vcpu>
              <os>
                <type arch="x86_64" machine="pc-q35-4.2">hvm</type>
                <loader readonly="yes" type="pflash">/usr/share/OVMF/OVMF_CODE.ms.fd</loader>
                <nvram>/var/lib/libvirt/qemu/nvram/{name}_VARS.fd</nvram>
                <kernel>{kernel_file}</kernel>
                <initrd>{initrd_file}</initrd>
                <cmdline>root=/dev/{root_disk_device_id} rw rootfstype={root_disk_type} systemd.unit=graphical.target</cmdline>
                <boot dev="hd"/>
              </os>
              <features>
                <acpi/>
                <apic/>
                <vmport state="off"/>
              </features>
              <cpu mode="host-model" check="partial"/>
              <clock offset="utc">
                <timer name="rtc" tickpolicy="catchup"/>
                <timer name="pit" tickpolicy="delay"/>
                <timer name="hpet" present="no"/>
              </clock>
              <on_poweroff>destroy</on_poweroff>
              <on_reboot>restart</on_reboot>
              <on_crash>destroy</on_crash>
              <pm>
                <suspend-to-mem enabled="no"/>
                <suspend-to-disk enabled="no"/>
              </pm>
              <devices>
                <emulator>/usr/bin/qemu-system-x86_64</emulator>
                {root_disk_xml}
                <controller type="usb" index="0" model="ich9-ehci1">
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x1d" function="0x7"/>
                </controller>
                <controller type="usb" index="0" model="ich9-uhci1">
                  <master startport="0"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x1d" function="0x0" multifunction="on"/>
                </controller>
                <controller type="usb" index="0" model="ich9-uhci2">
                  <master startport="2"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x1d" function="0x1"/>
                </controller>
                <controller type="usb" index="0" model="ich9-uhci3">
                  <master startport="4"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x1d" function="0x2"/>
                </controller>
                <controller type="sata" index="0">
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x1f" function="0x2"/>
                </controller>
                <controller type="pci" index="0" model="pcie-root"/>
                <controller type="pci" index="1" model="pcie-root-port">
                  <model name="pcie-root-port"/>
                  <target chassis="1" port="0x10"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x02" function="0x0" multifunction="on"/>
                </controller>
                <controller type="pci" index="2" model="pcie-root-port">
                  <model name="pcie-root-port"/>
                  <target chassis="2" port="0x11"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x02" function="0x1"/>
                </controller>
                <controller type="pci" index="3" model="pcie-root-port">
                  <model name="pcie-root-port"/>
                  <target chassis="3" port="0x12"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x02" function="0x2"/>
                </controller>
                <controller type="pci" index="4" model="pcie-root-port">
                  <model name="pcie-root-port"/>
                  <target chassis="4" port="0x13"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x02" function="0x3"/>
                </controller>
                <controller type="pci" index="5" model="pcie-root-port">
                  <model name="pcie-root-port"/>
                  <target chassis="5" port="0x14"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x02" function="0x4"/>
                </controller>
                <controller type="pci" index="6" model="pcie-root-port">
                  <model name="pcie-root-port"/>
                  <target chassis="6" port="0x15"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x02" function="0x5"/>
                </controller>
                <controller type="virtio-serial" index="0">
                  <address type="pci" domain="0x0000" bus="0x02" slot="0x00" function="0x0"/>
                </controller>
                <interface type="network">
                  <mac address="52:54:00:af:aa:a5"/>
                  <source network="default"/>
                  <model type="virtio"/>
                  <address type="pci" domain="0x0000" bus="0x01" slot="0x00" function="0x0"/>
                </interface>
                <serial type="pty">
                  <target type="isa-serial" port="0">
                    <model name="isa-serial"/>
                  </target>
                </serial>
                <console type="pty">
                  <target type="serial" port="0"/>
                </console>
                <channel type="unix">
                  <target type="virtio" name="org.qemu.guest_agent.0"/>
                  <address type="virtio-serial" controller="0" bus="0" port="1"/>
                </channel>
                <channel type="spicevmc">
                  <target type="virtio" name="com.redhat.spice.0"/>
                  <address type="virtio-serial" controller="0" bus="0" port="2"/>
                </channel>
                <input type="tablet" bus="usb">
                  <address type="usb" bus="0" port="1"/>
                </input>
                <input type="mouse" bus="ps2"/>
                <input type="keyboard" bus="ps2"/>
                <graphics type="spice" autoport="yes">
                  <listen type="address"/>
                  <image compression="off"/>
                </graphics>
                <sound model="ich9">
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x1b" function="0x0"/>
                </sound>
                <video>
                  <model type="virtio" heads="1" primary="yes"/>
                  <alias name="video0"/>
                  <address type="pci" domain="0x0000" bus="0x00" slot="0x01" function="0x0"/>
                </video>
                <redirdev bus="usb" type="spicevmc">
                  <address type="usb" bus="0" port="2"/>
                </redirdev>
                <redirdev bus="usb" type="spicevmc">
                  <address type="usb" bus="0" port="3"/>
                </redirdev>
                <memballoon model="virtio">
                  <address type="pci" domain="0x0000" bus="0x04" slot="0x00" function="0x0"/>
                </memballoon>
                <rng model="virtio">
                  <backend model="random">/dev/urandom</backend>
                  <address type="pci" domain="0x0000" bus="0x05" slot="0x00" function="0x0"/>
                </rng>
              </devices>
            </domain>
            "#,
            name = self.name,
            kernel_file = self.kernel_file,
            initrd_file = self.initrd_file,
            root_disk_xml = root_disk_xml,
            root_disk_type = root_filesystem_type,
            root_disk_device_id = root_disk_device_id,
            ram_in_bytes = self.ram_in_bytes,
            num_cpus = self.num_cpus
        ))
    }
}