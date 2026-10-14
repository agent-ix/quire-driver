// SPDX-License-Identifier: AGPL-3.0-or-later
//! Linux cgroup boundary for complete per-session resource accounting.

use process_wrap::std::ChildWrapper;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const SETUP_TIMEOUT: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(5);
static NEXT_UNIT: AtomicU64 = AtomicU64::new(1);

/// The transient systemd unit name and temporary environment file.
pub(super) struct ServiceHandle {
    pub(super) unit: String,
    environment_file: PathBuf,
}

impl ServiceHandle {
    /// Remove the private environment file once systemd has loaded the unit.
    pub(super) fn environment_loaded(&self) {
        let _ = fs::remove_file(&self.environment_file);
    }
}

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.environment_file);
    }
}

/// A complete cgroup-v2 accounting and cleanup boundary.
pub(super) struct Boundary {
    cgroup: PathBuf,
    cgroup_kill: File,
}

impl Boundary {
    /// Resolve a started transient service and prove CPU accounting and kill
    /// control are available before the caller sends a request.
    pub(super) fn attach(unit: String, memory_required: bool) -> io::Result<Self> {
        let until = Instant::now() + SETUP_TIMEOUT;
        loop {
            if let Ok(boundary) = Self::try_attach(&unit, memory_required) {
                return Ok(boundary);
            }
            if Instant::now() >= until {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "cgroup setup timed out",
                ));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn try_attach(unit: &str, memory_required: bool) -> io::Result<Self> {
        let main_pid = systemctl_show(unit, "ExecMainPID")?;
        if main_pid.parse::<u32>().ok().is_none_or(|pid| pid == 0) {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "transient service has not completed exec",
            ));
        }
        let control_group = systemctl_show(unit, "ControlGroup")?;
        let relative = control_group.trim_start_matches('/');
        if relative.is_empty()
            || Path::new(relative)
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(io::Error::other(
                "systemd returned an invalid control group path",
            ));
        }
        let cgroup = Path::new("/sys/fs/cgroup").join(relative);
        let usage = read_cpu_usage(&cgroup)?;
        let _ = usage;
        if memory_required {
            fs::read_to_string(cgroup.join("memory.current"))?
                .parse::<u64>()
                .map_err(|_| io::Error::other("cgroup memory.current is not a valid byte count"))?;
        }
        let cgroup_kill = OpenOptions::new()
            .write(true)
            .open(cgroup.join("cgroup.kill"))?;
        Ok(Self {
            cgroup,
            cgroup_kill,
        })
    }

    /// Aggregate CPU time from this cgroup and all descendant cgroups.
    pub(super) fn cpu_usage_micros(&self) -> io::Result<u64> {
        read_cpu_usage(&self.cgroup)
    }

    /// Current aggregate resident memory, when the caller requested it.
    pub(super) fn memory_current(&self) -> io::Result<u64> {
        fs::read_to_string(self.cgroup.join("memory.current"))?
            .trim()
            .parse::<u64>()
            .map_err(|_| io::Error::other("cgroup memory.current is not a valid byte count"))
    }

    /// Kill every cgroup member and wait for the service boundary and launcher
    /// to exit before the common cleanup deadline.
    pub(super) fn terminate(&mut self, child: &mut dyn ChildWrapper, deadline: Instant) -> bool {
        if child.try_wait().is_ok_and(|status| status.is_some())
            && !cgroup_populated(&self.cgroup).unwrap_or(true)
        {
            return true;
        }
        if self.cgroup_kill.write_all(b"1").is_err() || self.cgroup_kill.flush().is_err() {
            return false;
        }
        loop {
            let cgroup_empty = cgroup_populated(&self.cgroup).is_ok_and(|populated| !populated)
                || !self.cgroup.exists();
            let launcher_exited = child.try_wait().is_ok_and(|status| status.is_some());
            if cgroup_empty && launcher_exited {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(POLL_INTERVAL);
        }
    }
}

/// Build a transient service command without placing inherited environment
/// values on the command line. The environment file is private and removed as
/// soon as the service is visible in systemd.
pub(super) fn service_command(
    executable: &Path,
    arguments: &[OsString],
    working_directory: &Path,
) -> io::Result<(Command, ServiceHandle)> {
    let systemd_run = tool_path("systemd-run")?;
    let unit = format!(
        "quire-plugin-{}-{}.service",
        std::process::id(),
        NEXT_UNIT.fetch_add(1, Ordering::Relaxed)
    );
    let environment_file = write_environment_file()?;
    let mut command = Command::new(systemd_run);
    command
        .args([
            "--user",
            "--service-type=exec",
            "--pipe",
            "--wait",
            "--collect",
            "--quiet",
        ])
        .arg(format!("--unit={unit}"))
        .arg(format!(
            "--property=WorkingDirectory={}",
            path_text(working_directory)?
        ))
        .arg(format!(
            "--property=EnvironmentFile={}",
            path_text(&environment_file)?
        ))
        .args([
            "--property=KillMode=control-group",
            "--property=SendSIGKILL=yes",
            "--property=TimeoutStopSec=500ms",
        ])
        .arg("--")
        .arg(executable)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok((
        command,
        ServiceHandle {
            unit,
            environment_file,
        },
    ))
}

/// Stop a transient unit when boundary setup fails before request dispatch.
pub(super) fn stop_unit(unit: &str, deadline: Instant) -> bool {
    let Ok(systemctl) = tool_path("systemctl") else {
        return false;
    };
    let Ok(mut stop) = Command::new(systemctl)
        .args(["--user", "stop", unit])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    loop {
        match stop.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            _ => {
                let _ = stop.kill();
                let _ = stop.wait();
                return false;
            }
        }
    }
}

fn read_cpu_usage(cgroup: &Path) -> io::Result<u64> {
    let content = fs::read_to_string(cgroup.join("cpu.stat"))?;
    content
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(' ')?;
            (name == "usage_usec")
                .then(|| value.trim().parse::<u64>().ok())
                .flatten()
        })
        .ok_or_else(|| io::Error::other("cgroup cpu.stat has no valid usage_usec field"))
}

fn cgroup_populated(cgroup: &Path) -> io::Result<bool> {
    let content = fs::read_to_string(cgroup.join("cgroup.events"))?;
    content
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(' ')?;
            (name == "populated")
                .then(|| value.trim().parse::<u8>().ok())
                .flatten()
        })
        .map(|value| value != 0)
        .ok_or_else(|| io::Error::other("cgroup.events has no valid populated field"))
}

fn systemctl_show(unit: &str, property: &str) -> io::Result<String> {
    let systemctl = tool_path("systemctl")?;
    let mut child = Command::new(systemctl)
        .args([
            "--user",
            "show",
            "--no-pager",
            "--property",
            property,
            "--value",
            unit,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + SETUP_TIMEOUT;
    loop {
        match child.try_wait()? {
            Some(status) if status.success() => {
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    stdout.read_to_string(&mut output)?;
                }
                return Ok(output.trim().to_owned());
            }
            Some(_) => {
                return Err(io::Error::other(
                    "systemctl could not read the service cgroup",
                ))
            }
            None if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "systemctl setup timed out",
                ));
            }
        }
    }
}

fn tool_path(name: &str) -> io::Result<PathBuf> {
    [
        Path::new("/usr/bin").join(name),
        Path::new("/bin").join(name),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{name} is unavailable")))
}

fn path_text(path: &Path) -> io::Result<&str> {
    path.to_str()
        .ok_or_else(|| io::Error::other("systemd requires UTF-8 paths"))
}

fn write_environment_file() -> io::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!(
        "quire-plugin-env-{}-{}",
        std::process::id(),
        NEXT_UNIT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(&path)?;
    let result = (|| {
        for (key, value) in std::env::vars_os() {
            let key = key
                .to_str()
                .filter(|key| {
                    !key.is_empty()
                        && key
                            .chars()
                            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                })
                .ok_or_else(|| io::Error::other("plugin environment contains a non-text key"))?;
            let value = value
                .to_str()
                .ok_or_else(|| io::Error::other("plugin environment contains a non-text value"))?;
            writeln!(file, "{key}=\"{}\"", escape_environment_value(value))?;
        }
        file.sync_all()
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    Ok(path)
}

fn escape_environment_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{escape_environment_value, read_cpu_usage};
    use std::fs;

    /// Trace: FR-013-AC-8, TC-026; parse aggregate cgroup CPU use, including the exited-task total.
    #[test]
    fn cpu_stat_requires_and_parses_usage_usec() {
        let root = std::env::temp_dir().join(format!("quire-cgroup-stat-{}", std::process::id()));
        fs::create_dir_all(&root).expect("create cgroup fixture");
        fs::write(
            root.join("cpu.stat"),
            "usage_usec 1234\nuser_usec 800\nsystem_usec 434\n",
        )
        .expect("write cpu.stat fixture");
        assert_eq!(read_cpu_usage(&root).expect("parse aggregate CPU"), 1234);
        fs::write(root.join("cpu.stat"), "user_usec 800\nsystem_usec 434\n")
            .expect("write malformed cpu.stat fixture");
        assert!(read_cpu_usage(&root).is_err());
        fs::remove_dir_all(root).expect("remove cgroup fixture");
    }

    /// Trace: FR-013-AC-8, TC-026; encode environment values without command-line exposure.
    #[test]
    fn environment_file_escapes_systemd_quoted_values() {
        assert_eq!(
            escape_environment_value("a\\b\"c\nd\re\tf"),
            "a\\\\b\\\"c\\nd\\re\\tf"
        );
    }
}
