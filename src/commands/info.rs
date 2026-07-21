use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::tmux;
use crate::util::{home_dir, project_names};

pub fn show_info() -> Result<()> {
    system_info();
    println!();
    projects_info()?;

    if let Some(session) = tmux::current_session() {
        println!();
        project_info(&session)?;
    }

    Ok(())
}

fn system_info() {
    println!("system");
    let host = read_proc("/proc/sys/kernel/hostname").unwrap_or_else(|| "?".into());
    let ostype = read_proc("/proc/sys/kernel/ostype").unwrap_or_else(|| "?".into());
    let osrelease = read_proc("/proc/sys/kernel/osrelease").unwrap_or_default();

    row("host", &host);
    row("os", &format!("{ostype} {osrelease}").trim().to_string());
    if let Some(up) = uptime() {
        row("uptime", &up);
    }
    if let Some(load) = read_proc("/proc/loadavg") {
        let load: String = load.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
        row("load", &load);
    }
    if let Some(mem) = memory() {
        row("memory", &mem);
    }
}

fn projects_info() -> Result<()> {
    println!("projects");
    let projects = project_names()?;
    row("total", &projects.len().to_string());

    let active: Vec<String> = projects
        .iter()
        .filter(|name| tmux::session_exists(name).unwrap_or(false))
        .cloned()
        .collect();

    let active_desc = if active.is_empty() {
        "0".to_string()
    } else {
        format!("{} ({})", active.len(), active.join(", "))
    };
    row("active", &active_desc);
    Ok(())
}

fn project_info(session: &str) -> Result<()> {
    println!("project: {session}");
    let dir = home_dir()?.join(session);
    row("path", &dir.display().to_string());

    if dir.join(".git").is_dir() {
        if let Some(branch) = git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]) {
            row("branch", &branch);
        }
        let status = match git(&dir, &["status", "--porcelain"]) {
            Some(s) if !s.is_empty() => {
                let n = s.lines().count();
                format!("{n} file{} changed", if n == 1 { "" } else { "s" })
            }
            Some(_) => "clean".to_string(),
            None => "unknown".to_string(),
        };
        row("status", &status);
    }

    let windows = tmux::window_names(session);
    if !windows.is_empty() {
        row("windows", &windows.join(", "));
    }
    Ok(())
}

fn row(label: &str, value: &str) {
    println!("  {label:<9} {value}");
}

fn read_proc(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn uptime() -> Option<String> {
    let secs: f64 = read_proc("/proc/uptime")?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let secs = secs as u64;
    let (d, h, m) = (secs / 86400, (secs % 86400) / 3600, (secs % 3600) / 60);
    let mut parts = Vec::new();
    if d > 0 {
        parts.push(format!("{d}d"));
    }
    if h > 0 || d > 0 {
        parts.push(format!("{h}h"));
    }
    parts.push(format!("{m}m"));
    Some(parts.join(" "))
}

fn memory() -> Option<String> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let field = |key: &str| -> Option<u64> {
        meminfo
            .lines()
            .find(|l| l.starts_with(key))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    };
    let total_kb = field("MemTotal:")?;
    let avail_kb = field("MemAvailable:")?;
    let used_kb = total_kb.saturating_sub(avail_kb);
    let gib = |kb: u64| kb as f64 / 1024.0 / 1024.0;
    Some(format!("{:.1} / {:.1} GiB used", gib(used_kb), gib(total_kb)))
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
