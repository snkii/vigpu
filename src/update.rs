// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-13)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[cfg(unix)]
use std::thread;
use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime},
};

use sha2::{Digest, Sha256};

const RELEASE_API: &str = "https://api.github.com/repos/snkii/vigpu/releases/latest";
const RELEASE_ASSET_PREFIX: &str = "https://github.com/snkii/vigpu/releases/download/";
const WORKER_ARG: &str = "--__vigpu-update-worker";

struct Asset {
    url: String,
    digest: String,
    archive_name: &'static str,
    binary_name: &'static str,
}

pub fn handle_internal_args() -> bool {
    let mut args = env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new(WORKER_ARG)) {
        return false;
    }
    if let Some(pid) = args
        .next()
        .and_then(|value| value.to_string_lossy().parse().ok())
    {
        let _ = update_worker(pid);
    }
    true
}

pub fn dispatch_user_overlay() {
    let Some(overlay) = overlay_path() else {
        return;
    };
    if !overlay.exists() || !overlay_active_path(&overlay).exists() {
        return;
    }
    let Ok(current) = env::current_exe() else {
        return;
    };
    if same_path(&current, &overlay) {
        return;
    }
    let mut command = Command::new(overlay);
    command.args(env::args_os().skip(1));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = command.exec();
        let _ = error;
    }
    #[cfg(windows)]
    {
        if let Ok(status) = command.status() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}

pub fn spawn_background_check() {
    let Ok(executable) = env::current_exe() else {
        return;
    };
    let mut command = Command::new(executable);
    command
        .arg(WORKER_ARG)
        .arg(std::process::id().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0000_0008 | 0x0800_0000);
    }
    let _ = command.spawn();
}

fn update_worker(parent_pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut _lock) = WorkerLock::acquire()? else {
        return Ok(());
    };
    let executable = env::current_exe()?;
    let release_asset = match latest_asset()? {
        Some(asset) => asset,
        None => return Ok(()),
    };
    let executable_marker = marker_path(&executable)?;
    if fs::read_to_string(&executable_marker).is_ok_and(|value| {
        let Some((digest, binary_hash)) = value.trim().split_once(':') else {
            return false;
        };
        digest == release_asset.digest
            && hash_file(&executable).is_ok_and(|current| current == binary_hash)
    }) {
        return Ok(());
    }

    let temp = TempDir::create()?;
    let archive = temp.0.join(release_asset.archive_name);
    if !download(&release_asset.url, &archive, 60)? {
        return Ok(());
    }
    if hash_file(&archive)? != release_asset.digest {
        return Ok(());
    }
    let extracted = temp.0.join("extracted");
    fs::create_dir_all(&extracted)?;
    if !extract(&archive, &extracted)? {
        return Ok(());
    }
    let new_binary = extracted.join(release_asset.binary_name);
    if !new_binary.is_file() {
        return Ok(());
    }
    let new_binary_hash = hash_file(&new_binary)?;
    let installed_fingerprint = format!("{}:{new_binary_hash}", release_asset.digest);
    if hash_file(&executable).is_ok_and(|hash| hash == new_binary_hash) {
        write_marker(&executable_marker, &installed_fingerprint)?;
        return Ok(());
    }

    let target = if writable_parent(&executable) {
        executable.clone()
    } else {
        let Some(overlay) = overlay_path() else {
            return Ok(());
        };
        overlay
    };
    let target_parent = target
        .parent()
        .ok_or_else(|| io::Error::other("Update target has no parent directory"))?;
    fs::create_dir_all(target_parent)?;
    let staged = target_parent.join(format!(".vigpu-update-{}.new", std::process::id()));
    fs::copy(&new_binary, &staged)?;
    set_executable(&staged)?;
    let marker = marker_path(&target)?;
    if let Some(parent) = marker.parent() {
        fs::create_dir_all(parent)?;
    }
    let active = (target != executable).then(|| overlay_active_path(&target));

    #[cfg(unix)]
    {
        wait_for_parent(parent_pid);
        fs::rename(&staged, &target)?;
        write_marker(&marker, &installed_fingerprint)?;
        if let Some(active) = active {
            write_marker(&active, "active")?;
        }
    }
    #[cfg(windows)]
    {
        let lock_path = _lock.path.clone();
        spawn_windows_apply(
            parent_pid,
            &staged,
            &target,
            &marker,
            active.as_deref(),
            &installed_fingerprint,
            &lock_path,
            &temp.0,
        )?;
        _lock.keep = true;
    }
    Ok(())
}

fn latest_asset() -> Result<Option<Asset>, Box<dyn std::error::Error>> {
    let cache = cache_dir()?;
    fs::create_dir_all(&cache)?;
    let body_path = cache.join("latest-release.json");
    let etag_path = cache.join("latest-release.etag");
    let response_path = cache.join(format!("latest-release-{}.tmp", std::process::id()));
    let curl = if cfg!(windows) { "curl.exe" } else { "curl" };
    let mut command = Command::new(curl);
    command.args([
        "--location",
        "--silent",
        "--show-error",
        "--connect-timeout",
        "2",
        "--max-time",
        "5",
        "--proto-redir",
        "=https",
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        "User-Agent: ViGPU",
        "--output",
    ]);
    command
        .arg(&response_path)
        .args(["--write-out", "%{http_code}"]);
    if etag_path.is_file() {
        command.arg("--etag-compare").arg(&etag_path);
        command.arg("--etag-save").arg(&etag_path);
    } else {
        command.arg("--etag-save").arg(&etag_path);
    }
    let output = match command.arg(RELEASE_API).output() {
        Ok(output) if output.status.success() => output,
        _ => {
            let _ = fs::remove_file(&response_path);
            return Ok(None);
        }
    };
    let status = String::from_utf8_lossy(&output.stdout);
    match status.trim() {
        "200" => {
            let body = fs::read(&response_path)?;
            fs::write(&body_path, body)?;
        }
        "304" if body_path.is_file() => {}
        _ => {
            let _ = fs::remove_file(&response_path);
            return Ok(None);
        }
    }
    let _ = fs::remove_file(&response_path);
    let release: serde_json::Value = serde_json::from_slice(&fs::read(body_path)?)?;
    let asset_name = platform_asset_names();
    let Some((archive_name, binary_name)) = asset_name else {
        return Ok(None);
    };
    let Some(assets) = release.get("assets").and_then(serde_json::Value::as_array) else {
        return Ok(None);
    };
    let Some(asset) = assets
        .iter()
        .find(|asset| asset.get("name").and_then(serde_json::Value::as_str) == Some(archive_name))
    else {
        return Ok(None);
    };
    let Some(url) = asset
        .get("browser_download_url")
        .and_then(serde_json::Value::as_str)
        .filter(|url| url.starts_with(RELEASE_ASSET_PREFIX))
    else {
        return Ok(None);
    };
    let Some(digest) = asset
        .get("digest")
        .and_then(serde_json::Value::as_str)
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
    else {
        return Ok(None);
    };
    Ok(Some(Asset {
        url: url.to_owned(),
        digest: digest.to_ascii_lowercase(),
        archive_name,
        binary_name,
    }))
}

fn platform_asset_names() -> Option<(&'static str, &'static str)> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return Some(("vigpu-linux-x64.tar.gz", "vigpu"));
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Some(("vigpu-macos-arm64.tar.gz", "vigpu"));
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Some(("vigpu-windows-x64.zip", "vigpu.exe"));
    #[allow(unreachable_code)]
    None
}

fn download(url: &str, destination: &Path, timeout: u64) -> io::Result<bool> {
    if !url.starts_with(RELEASE_ASSET_PREFIX) {
        return Ok(false);
    }
    let curl = if cfg!(windows) { "curl.exe" } else { "curl" };
    let output = Command::new(curl)
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--connect-timeout",
            "3",
            "--max-time",
        ])
        .arg(timeout.to_string())
        .args(["--proto-redir", "=https", "--output"])
        .arg(destination)
        .arg(url)
        .output()?;
    Ok(output.status.success())
}

fn extract(archive: &Path, destination: &Path) -> io::Result<bool> {
    #[cfg(windows)]
    {
        let script = format!(
            "$ErrorActionPreference='Stop'; Expand-Archive -LiteralPath {} -DestinationPath {} -Force",
            ps_quote(archive),
            ps_quote(destination)
        );
        return Ok(Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success());
    }
    #[cfg(not(windows))]
    {
        Ok(Command::new("tar")
            .args(["-xzf"])
            .arg(archive)
            .args(["-C"])
            .arg(destination)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success())
    }
}

fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 32 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn marker_path(executable: &Path) -> io::Result<PathBuf> {
    let key = hex(&Sha256::digest(executable.to_string_lossy().as_bytes()));
    Ok(cache_dir()?.join("installed").join(key))
}

fn write_marker(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temporary, contents)?;
    fs::rename(&temporary, path).or_else(|error| {
        if path.exists() {
            fs::remove_file(path)?;
            fs::rename(&temporary, path)
        } else {
            Err(error)
        }
    })
}

fn cache_dir() -> io::Result<PathBuf> {
    #[cfg(windows)]
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is unavailable"))?
        .join("ViGPU")
        .join("Cache");
    #[cfg(not(windows))]
    let base = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory is unavailable"))?
        .join("vigpu");
    Ok(base)
}

fn overlay_path() -> Option<PathBuf> {
    #[cfg(windows)]
    return env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|base| base.join("ViGPU").join("Updates").join("vigpu.exe"));
    #[cfg(not(windows))]
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .map(|base| base.join("vigpu").join("vigpu"))
}

fn overlay_active_path(executable: &Path) -> PathBuf {
    executable.with_extension("active")
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn writable_parent(executable: &Path) -> bool {
    let Some(parent) = executable.parent() else {
        return false;
    };
    let probe = parent.join(format!(".vigpu-write-test-{}", std::process::id()));
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(file) => {
            drop(file);
            let _ = fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn set_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[cfg(unix)]
fn wait_for_parent(parent_pid: u32) {
    while unsafe { libc::kill(parent_pid as libc::pid_t, 0) } == 0
        || io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    {
        thread::sleep(Duration::from_millis(300));
    }
}

#[cfg(windows)]
fn spawn_windows_apply(
    parent_pid: u32,
    staged: &Path,
    target: &Path,
    marker: &Path,
    active: Option<&Path>,
    fingerprint: &str,
    lock: &Path,
    temp_dir: &Path,
) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    let worker_pid = std::process::id();
    let script_path = temp_dir.join("apply-update.ps1");
    let active_command = active
        .map(|path| format!("[IO.File]::WriteAllText({}, 'active')", ps_quote(path)))
        .unwrap_or_else(|| "$true".to_owned());
    let script = format!(
        "$ErrorActionPreference='Stop'; function Wait-Exit([int]$id) {{ while (Get-Process -Id $id -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 300 }} }}; Wait-Exit {parent_pid}; Wait-Exit {worker_pid}; $target={target}; $staged={staged}; $backup=$target+'.vigpu-old'; $done=$false; for($i=0; $i -lt 120 -and -not $done; $i++) {{ try {{ if(Test-Path -LiteralPath $target) {{ Move-Item -LiteralPath $target -Destination $backup -Force }}; Move-Item -LiteralPath $staged -Destination $target -Force; if(Test-Path -LiteralPath $backup) {{ Remove-Item -LiteralPath $backup -Force }}; [IO.File]::WriteAllText({marker}, {fingerprint}); {active_command}; $done=$true }} catch {{ if((Test-Path -LiteralPath $backup) -and -not (Test-Path -LiteralPath $target)) {{ Move-Item -LiteralPath $backup -Destination $target -Force }}; Start-Sleep -Milliseconds 500 }} }}; Remove-Item -LiteralPath {lock} -Force -ErrorAction SilentlyContinue; Remove-Item -LiteralPath {script} -Force -ErrorAction SilentlyContinue; Remove-Item -LiteralPath {temp_dir} -Recurse -Force -ErrorAction SilentlyContinue",
        target = ps_quote(target),
        staged = ps_quote(staged),
        marker = ps_quote(marker),
        fingerprint = ps_quote_value(fingerprint),
        lock = ps_quote(lock),
        script = ps_quote(&script_path),
        temp_dir = ps_quote(temp_dir),
    );
    fs::write(&script_path, script)?;
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&script_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x0000_0008 | 0x0800_0000)
        .spawn()?;
    drop(status);
    Ok(())
}

#[cfg(windows)]
fn ps_quote(path: &Path) -> String {
    ps_quote_value(&path.to_string_lossy())
}

#[cfg(windows)]
fn ps_quote_value(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

struct TempDir(PathBuf);

impl TempDir {
    fn create() -> io::Result<Self> {
        for attempt in 0..8 {
            let path = env::temp_dir().join(format!(
                "vigpu-update-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
                attempt
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Could not create update staging directory",
        ))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        #[cfg(not(windows))]
        let _ = fs::remove_dir_all(&self.0);
        #[cfg(windows)]
        if !self.0.join("apply-update.ps1").exists() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

struct WorkerLock {
    path: PathBuf,
    keep: bool,
}

impl WorkerLock {
    fn acquire() -> io::Result<Option<Self>> {
        let path = cache_dir()?.join("update-worker.lock");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                writeln!(file, "{}", std::process::id())?;
                Ok(Some(Self { path, keep: false }))
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let stale = fs::metadata(&path)
                    .and_then(|metadata| metadata.modified())
                    .ok()
                    .and_then(|modified| modified.elapsed().ok())
                    .is_some_and(|age| age > Duration::from_secs(15 * 60));
                if stale {
                    let _ = fs::remove_file(&path);
                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&path)
                    {
                        Ok(mut file) => {
                            writeln!(file, "{}", std::process::id())?;
                            Ok(Some(Self { path, keep: false }))
                        }
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(None),
                        Err(error) => Err(error),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(error) => Err(error),
        }
    }
}

impl Drop for WorkerLock {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_file(&self.path);
        }
    }
}
