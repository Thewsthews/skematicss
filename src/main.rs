use chrono::Duration;
use std::collections::HashMap;
use std::fmt::Debug;
use std::process::ExitCode;
use sysinfo::{DiskExt, NetworkExt, System };
use sysinfo::SystemExt;
use sysinfo::ComponentExt;

const LOGO_HEIGHT: usize = 9;
const LOGO_WIDTH: usize = 32;
const LOGO: [&str; LOGO_HEIGHT] = [
      "                               ",
    "    #       #      *#*         ",
    "    ##     ##      *#*         ",
    "    #  # #  #      *#*         ",
    "    #   #   #      *#*         ",
    "    #       #      *#*         ",
    "    #       #      *#*         ",
    "    #       #      *#*         ",
    "                               ",
];

struct DiskInfo{
    mount_point: String,
    total_space_gb: u64,
    used_space_gb: u64,
    filesystem: String,
}

impl Debug for DiskInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        f.debug_struct("DiskInfo")
            .field("mount_point", &self.mount_point)
            .field("total_space_gb", &self.total_space_gb)
            .field("used_space_gb", &self.used_space_gb)
            .field("filesystem", &self.filesystem)
            .finish()
    }
}

struct TemperatureInfo{
    label: String,
    temperature_celsius: f32,
}

impl Debug for TemperatureInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        f.debug_struct("TemperatureInfo")
            .field("label", &self.label)
            .field("temperature_celsius", &self.temperature_celsius)
            .finish()
    }
}

struct NetworkInfo{
    interface_name: String,
    bytes_sent: u64,
    bytes_received: u64,
    packets_sent: u64,
    packets_received: u64,
}

impl Debug for NetworkInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        f.debug_struct("NetworkInfo")
            .field("interface_name", &self.interface_name)
            .field("bytes_sent", &self.bytes_sent)
            .field("bytes_received", &self.bytes_received)
            .field("packets_sent", &self.packets_sent)
            .field("packets_received", &self.packets_received)
            .finish()
    }
}

struct OutputInfo<'a>{
    username: String,
    hostname: String,
    os: String,
    kernel: String,
    uptime: usize,
    disks: HashMap<String, DiskInfo>,
    temperatures: HashMap<String, TemperatureInfo>,
    networks: HashMap<&'a str, NetworkInfo>,
}

fn get_username() -> String{
    whoami::username()
}

fn get_hostname() -> String{
    whoami::fallible::hostname().unwrap_or(String::from("unknown"))
}

fn get_os_name() -> String{
    whoami::distro()
}

fn kernel(sys: &System) -> String {
    sys.kernel_version().unwrap_or_else(|| String::from("unknown"))
}

fn get_uptime(sys: &System) -> usize{
    sys.uptime() as usize
}

fn get_disk_info(sys: &System) -> HashMap<String, DiskInfo> {
    let mut disk_info_map = HashMap::new();
    for disk in sys.disks() {
        let mount_point = disk.mount_point().to_string_lossy().into_owned();
        disk_info_map.insert(
            mount_point.clone(),
            DiskInfo {
                mount_point: mount_point.clone(),
                total_space_gb: disk.total_space() / 1024 / 1024 / 1024,
                used_space_gb: (disk.total_space() - disk.available_space()) / 1024 / 1024 / 1024,
                filesystem: String::from_utf8_lossy(disk.file_system()).into_owned(),
            },
        );
    }
    disk_info_map
}

fn get_temperature_info(sys: &System) -> HashMap<String, TemperatureInfo> {
    let mut temp_info_map = HashMap::new();
    for component in sys.components() {
        let label = component.label().to_string();
        temp_info_map.insert(
            label.clone(),
            TemperatureInfo {
                label: label.clone(),
                temperature_celsius: component.temperature(),
            },
        );
    }
    temp_info_map
}

fn get_network_info<'a>(sys: &'a System) -> HashMap<&'a str, NetworkInfo>{
    let mut network_info_map = HashMap::new();
    for (interface_name, network) in sys.networks() {
        network_info_map.insert(
            interface_name.as_str(),
            NetworkInfo {
                interface_name: interface_name.to_string(),
                bytes_sent: network.total_transmitted(),
                bytes_received: network.total_received(),
                packets_sent: network.packets_transmitted(),
                packets_received: network.packets_received(),
            },
        );
    }
    network_info_map
}

fn convert_unix_to_human_string(unix_time: usize) -> String{
    let duration = Duration::seconds(unix_time as i64);
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn print_all_info(output_info: &OutputInfo) {
    let mut output_info_vec = vec![
        format!("{}@{}", output_info.username, output_info.hostname),
        format!("{}", "-".repeat(output_info.username.len() + output_info.hostname.len() + 1)),
        format!("OS:        {}", output_info.os),
        format!("Kernel:    {}", output_info.kernel),
        format!("Uptime:    {}", convert_unix_to_human_string(output_info.uptime)),
    ];
    for (mount_point, disk_info) in &output_info.disks {
        output_info_vec.push(format!(
            "Disk:      {} - {} GB used / {} GB total, FS: {}",
            mount_point, disk_info.used_space_gb, disk_info.total_space_gb, disk_info.filesystem
        ));
    }
    for (label, temp_info) in &output_info.temperatures {
        output_info_vec.push(format!(
            "Temp:      {} - {:.1}°C",
            label, temp_info.temperature_celsius
        ));
    }
    for (interface_name, network_info) in &output_info.networks {
        output_info_vec.push(format!(
            "Network:   {} - {} MB sent, {} MB recv, {} pkts sent, {} pkts recv",
            interface_name,
            network_info.bytes_sent / 1024 / 1024,
            network_info.bytes_received / 1024 / 1024,
            network_info.packets_sent,
            network_info.packets_received
        ));
    }
    println!();
    for (idx, line) in output_info_vec.iter().enumerate() {
        if idx < LOGO_HEIGHT {
            println!("{}{}", LOGO[idx], line);
        } else {
            println!("{}{}", " ".repeat(LOGO_WIDTH), line);
        }
    }
    if output_info_vec.len() < LOGO_HEIGHT {
        for i in output_info_vec.len()..LOGO_HEIGHT {
            println!("{}", LOGO[i]);
        }
    }
    println!();
}

fn main() -> ExitCode{
    // Removed unsupported sysinfo::IS_SUPPORTED_SYSTEM check

    let mut sys = System::new_all();
    sys.refresh_disks();
    sys.refresh_components();
    sys.refresh_networks();

    let output_info = OutputInfo {
        username: get_username(),
        hostname: get_hostname(),
        os: get_os_name(),
        kernel: kernel(&sys),
        uptime: get_uptime(&sys),
        disks: get_disk_info(&sys),
        temperatures: get_temperature_info(&sys),
        networks: get_network_info(&sys),
    };

    print_all_info(&output_info);

    ExitCode::from(0)
}