// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-12)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(not(target_os = "macos"))]
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[cfg(not(target_os = "macos"))]
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, enums::device::UsedGpuMemory, Nvml};

#[derive(Clone, Default)]
pub struct GpuSample {
    pub index: u32,
    pub name: String,
    pub utilization: Option<u32>,
    pub memory_utilization: Option<u32>,
    pub memory_used: Option<u64>,
    pub memory_total: Option<u64>,
    pub unified_memory: bool,
    pub temperature: Option<u32>,
    pub power_w: Option<f64>,
    pub power_limit_w: Option<f64>,
    pub fan_percent: Option<u32>,
}

#[derive(Clone)]
pub struct GpuProcess {
    pub gpu: u32,
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub memory_bytes: Option<u64>,
    pub cpu_percent: Option<f64>,
}

#[derive(Default)]
pub struct Snapshot {
    pub gpus: Vec<GpuSample>,
    pub processes: Vec<GpuProcess>,
}

#[cfg(not(target_os = "macos"))]
pub struct Collector {
    nvml: Nvml,
    previous_process_ticks: HashMap<u32, u64>,
    previous_sample: Option<Instant>,
    clock_ticks_per_second: Option<f64>,
}

#[cfg(target_os = "macos")]
pub struct Collector {
    samples: std::sync::mpsc::Receiver<Result<Snapshot, String>>,
}

#[cfg(not(target_os = "macos"))]
impl Collector {
    pub fn new() -> Result<Self, String> {
        let nvml = Nvml::init().map_err(|error| format!("NVML initialization failed: {error}"))?;
        Ok(Self {
            nvml,
            previous_process_ticks: HashMap::new(),
            previous_sample: None,
            clock_ticks_per_second: clock_ticks_per_second(),
        })
    }

    pub fn sample(&mut self) -> Result<Snapshot, String> {
        let sampled_at = Instant::now();
        let elapsed = self
            .previous_sample
            .map(|previous| sampled_at.saturating_duration_since(previous));
        let count = self
            .nvml
            .device_count()
            .map_err(|error| error.to_string())?;
        let mut snapshot = Snapshot::default();
        let mut identities = HashMap::new();
        let mut cpu_ticks = HashMap::new();

        for index in 0..count {
            let device = match self.nvml.device_by_index(index) {
                Ok(device) => device,
                Err(_) => continue,
            };
            let memory = device.memory_info().ok();
            let utilization = device.utilization_rates().ok();
            snapshot.gpus.push(GpuSample {
                index,
                name: device.name().unwrap_or_else(|_| "NVIDIA GPU".into()),
                utilization: utilization.as_ref().map(|value| value.gpu),
                memory_utilization: utilization.as_ref().map(|value| value.memory),
                memory_used: memory.as_ref().map(|value| value.used),
                memory_total: memory.as_ref().map(|value| value.total),
                unified_memory: false,
                temperature: device.temperature(TemperatureSensor::Gpu).ok(),
                power_w: device.power_usage().ok().map(|mw| mw as f64 / 1000.0),
                power_limit_w: device
                    .power_management_limit()
                    .ok()
                    .map(|mw| mw as f64 / 1000.0),
                fan_percent: device.fan_speed(0).ok(),
            });

            if let Ok(processes) = device.running_compute_processes() {
                for process in processes {
                    let (name, user) = process_identity(process.pid, &mut identities);
                    let ticks = *cpu_ticks
                        .entry(process.pid)
                        .or_insert_with(|| process_cpu_ticks(process.pid));
                    snapshot.processes.push(GpuProcess {
                        gpu: index,
                        pid: process.pid,
                        name,
                        user,
                        memory_bytes: used_bytes(process.used_gpu_memory),
                        cpu_percent: cpu_percent(
                            ticks,
                            self.previous_process_ticks.get(&process.pid).copied(),
                            elapsed,
                            self.clock_ticks_per_second,
                        ),
                    });
                }
            }
            if let Ok(processes) = device.running_graphics_processes() {
                for process in processes {
                    if snapshot
                        .processes
                        .iter()
                        .any(|entry| entry.gpu == index && entry.pid == process.pid)
                    {
                        continue;
                    }
                    let (name, user) = process_identity(process.pid, &mut identities);
                    let ticks = *cpu_ticks
                        .entry(process.pid)
                        .or_insert_with(|| process_cpu_ticks(process.pid));
                    snapshot.processes.push(GpuProcess {
                        gpu: index,
                        pid: process.pid,
                        name,
                        user,
                        memory_bytes: used_bytes(process.used_gpu_memory),
                        cpu_percent: cpu_percent(
                            ticks,
                            self.previous_process_ticks.get(&process.pid).copied(),
                            elapsed,
                            self.clock_ticks_per_second,
                        ),
                    });
                }
            }
        }
        snapshot.processes.sort_by(|a, b| {
            a.gpu
                .cmp(&b.gpu)
                .then_with(|| {
                    b.memory_bytes
                        .unwrap_or(0)
                        .cmp(&a.memory_bytes.unwrap_or(0))
                })
                .then_with(|| a.pid.cmp(&b.pid))
        });
        self.previous_process_ticks = cpu_ticks
            .into_iter()
            .filter_map(|(pid, ticks)| ticks.map(|ticks| (pid, ticks)))
            .collect();
        self.previous_sample = Some(sampled_at);
        Ok(snapshot)
    }
}

#[cfg(target_os = "macos")]
impl Collector {
    pub fn new() -> Result<Self, String> {
        let (sender, samples) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut sampler = match macmon::Sampler::new() {
                Ok(sampler) => sampler,
                Err(error) => {
                    let _ = sender.send(Err(format!("Apple GPU telemetry unavailable: {error}")));
                    return;
                }
            };
            let name = format!("{} GPU", sampler.get_soc_info().chip_name);
            loop {
                let metrics = match sampler.get_metrics(250) {
                    Ok(metrics) => metrics,
                    Err(error) => {
                        let _ = sender.send(Err(format!("Apple GPU telemetry failed: {error}")));
                        break;
                    }
                };
                let fan_percent = metrics.fans.first().and_then(|fan| {
                    fan.max_rpm
                        .filter(|maximum| *maximum > 0)
                        .map(|maximum| (fan.rpm * 100 / maximum).min(100))
                });
                let snapshot = Snapshot {
                    gpus: vec![GpuSample {
                        index: 0,
                        name: name.clone(),
                        utilization: Some((metrics.gpu_scaled_ratio * 100.0).round() as u32),
                        memory_utilization: None,
                        memory_used: Some(metrics.memory.ram_usage),
                        memory_total: Some(metrics.memory.ram_total),
                        unified_memory: true,
                        temperature: metrics.temp.gpu_temp_avg.map(|value| value.round() as u32),
                        power_w: Some(f64::from(metrics.gpu_power)),
                        power_limit_w: None,
                        fan_percent,
                    }],
                    processes: Vec::new(),
                };
                if sender.send(Ok(snapshot)).is_err() {
                    break;
                }
            }
        });
        Ok(Self { samples })
    }

    pub fn sample(&mut self) -> Result<Snapshot, String> {
        self.samples
            .recv()
            .map_err(|error| format!("Apple GPU telemetry stopped: {error}"))?
    }
}

#[cfg(target_os = "linux")]
fn process_cpu_ticks(pid: u32) -> Option<u64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields = stat.rsplit_once(')')?.1.split_whitespace();
    let mut fields = fields.skip(11);
    let user_ticks = fields.next()?.parse::<u64>().ok()?;
    let system_ticks = fields.next()?.parse::<u64>().ok()?;
    Some(user_ticks + system_ticks)
}

#[cfg(windows)]
fn process_cpu_ticks(pid: u32) -> Option<u64> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, FILETIME},
        System::Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    };

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return None;
    }
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    let ok = unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) };
    unsafe { CloseHandle(process) };
    if ok == 0 {
        return None;
    }
    let ticks =
        |time: FILETIME| (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
    Some(ticks(kernel) + ticks(user))
}

#[cfg(all(not(target_os = "linux"), not(windows), not(target_os = "macos")))]
fn process_cpu_ticks(_: u32) -> Option<u64> {
    None
}

#[cfg(not(target_os = "macos"))]
fn cpu_percent(
    current_ticks: Option<u64>,
    previous_ticks: Option<u64>,
    elapsed: Option<Duration>,
    ticks_per_second: Option<f64>,
) -> Option<f64> {
    let delta_ticks = current_ticks?.checked_sub(previous_ticks?)?;
    let elapsed_seconds = elapsed?.as_secs_f64();
    let ticks_per_second = ticks_per_second?;
    (elapsed_seconds > 0.0 && ticks_per_second > 0.0)
        .then(|| delta_ticks as f64 / ticks_per_second / elapsed_seconds * 100.0)
}

#[cfg(not(target_os = "macos"))]
fn clock_ticks_per_second() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        (ticks > 0).then_some(ticks as f64)
    }
    #[cfg(not(target_os = "linux"))]
    {
        {
            Some(10_000_000.0)
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn used_bytes(value: UsedGpuMemory) -> Option<u64> {
    match value {
        UsedGpuMemory::Used(bytes) => Some(bytes),
        UsedGpuMemory::Unavailable => None,
    }
}

#[cfg(not(target_os = "macos"))]
fn process_identity(pid: u32, cache: &mut HashMap<u32, (String, String)>) -> (String, String) {
    cache
        .entry(pid)
        .or_insert_with(|| (process_name(pid), process_user(pid)))
        .clone()
}

#[cfg(all(target_os = "linux", not(target_os = "macos")))]
fn process_name(pid: u32) -> String {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|name| name.trim().to_owned())
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("pid {pid}"));
    let Ok(command_line) = fs::read(format!("/proc/{pid}/cmdline")) else {
        return comm;
    };
    let args: Vec<String> = command_line
        .split(|byte| *byte == 0)
        .filter(|arg| !arg.is_empty())
        .map(|arg| String::from_utf8_lossy(arg).into_owned())
        .collect();
    if args.is_empty() {
        return comm;
    }
    let command = args.join(" ");
    let command: String = command
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();
    let command = command.split_whitespace().collect::<Vec<_>>().join(" ");
    if command.is_empty() {
        comm
    } else {
        command
    }
}

#[cfg(all(windows, not(target_os = "macos")))]
fn process_name(pid: u32) -> String {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        },
    };

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return format!("pid {pid}");
    }
    let mut buffer = vec![0u16; 32_768];
    let mut length = buffer.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) };
    unsafe { CloseHandle(process) };
    if ok == 0 || length == 0 {
        return format!("pid {pid}");
    }
    let path = String::from_utf16_lossy(&buffer[..length as usize]);
    std::path::Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("pid {pid}"))
}

#[cfg(all(not(target_os = "linux"), not(windows), not(target_os = "macos")))]
fn process_name(pid: u32) -> String {
    format!("pid {pid}")
}

#[cfg(all(target_os = "linux", not(target_os = "macos")))]
fn process_user(pid: u32) -> String {
    let uid = fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("Uid:")?
                    .split_whitespace()
                    .next()?
                    .parse::<u32>()
                    .ok()
            })
        });
    let Some(uid) = uid else {
        return "?".to_owned();
    };
    fs::read_to_string("/etc/passwd")
        .ok()
        .and_then(|passwd| {
            passwd.lines().find_map(|line| {
                let mut fields = line.split(':');
                let name = fields.next()?;
                let _password = fields.next()?;
                let entry_uid = fields.next()?.parse::<u32>().ok()?;
                (entry_uid == uid).then(|| name.to_owned())
            })
        })
        .unwrap_or_else(|| uid.to_string())
}

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
fn process_user(_: u32) -> String {
    "?".to_owned()
}
