use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::env;
use std::path::Path;
use std::process::Command;

/// Each project workspace gets these tabs, in order: (label, startup command).
/// The first reuses the workspace's own default tab; the rest are created fresh.
/// An empty command just leaves a bare shell.
const TABS: &[(&str, &str)] = &[
    ("claude", "claude"),
    ("agent", "agent"),
    ("remote", "claude remote-control"),
    ("shell", ""),
];

/// Open a project in herdr: focus its workspace if one already exists, otherwise
/// build a fresh one (four tabs, each running its agent) and land on `shell`.
pub fn open(name: &str, dir: &Path) -> Result<()> {
    let dir = dir.to_str().context("project path contains invalid UTF-8")?;

    if let Some(id) = workspace_id(name)? {
        eprintln!("herdr workspace '{name}' already exists — focusing");
        return run(&["workspace", "focus", &id]);
    }

    create_workspace(name, dir)
}

fn create_workspace(name: &str, dir: &str) -> Result<()> {
    // A new workspace comes with one default tab and root pane; reuse those for
    // the first entry, then create the remaining tabs.
    let ws = query(&["workspace", "create", "--label", name, "--cwd", dir])?;
    let ws_id = string(&ws, &["workspace", "workspace_id"])?;

    let mut shell_tab = String::new();
    for (i, (label, cmd)) in TABS.iter().enumerate() {
        let (tab_id, pane_id) = if i == 0 {
            let tab_id = string(&ws, &["tab", "tab_id"])?;
            run(&["tab", "rename", &tab_id, label])?;
            (tab_id, string(&ws, &["root_pane", "pane_id"])?)
        } else {
            let tab = query(&[
                "tab", "create", "--workspace", &ws_id, "--label", label, "--cwd", dir,
            ])?;
            (
                string(&tab, &["tab", "tab_id"])?,
                string(&tab, &["root_pane", "pane_id"])?,
            )
        };

        if !cmd.is_empty() {
            run(&["pane", "run", &pane_id, cmd])?;
        }
        if *label == "shell" {
            shell_tab = tab_id;
        }
    }

    // Land focused on the shell tab, matching the old select-window + attach.
    run(&["workspace", "focus", &ws_id])?;
    run(&["tab", "focus", &shell_tab])
}

/// Label of the workspace we're currently inside, if any.
pub fn current_workspace() -> Option<String> {
    let id = env::var("HERDR_WORKSPACE_ID").ok()?;
    workspaces().ok()?.into_iter().find(|w| w.id == id).map(|w| w.label)
}

/// Tab labels for a project's workspace, in order (empty if the lookup fails).
pub fn tab_names(name: &str) -> Vec<String> {
    let Ok(Some(id)) = workspace_id(name) else {
        return Vec::new();
    };
    let Ok(list) = query(&["tab", "list", "--workspace", &id]) else {
        return Vec::new();
    };
    list.get("tabs")
        .and_then(Value::as_array)
        .map(|tabs| {
            tabs.iter()
                .filter_map(|t| t.get("label").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Close a project's workspace and everything running in it.
pub fn close_workspace(name: &str) -> Result<()> {
    let id = workspace_id(name)?
        .with_context(|| format!("no active workspace for '{name}'"))?;
    run(&["workspace", "close", &id])
}

/// Labels of all live workspaces (empty if the lookup fails or the server is down).
pub fn workspace_names() -> Vec<String> {
    workspaces()
        .map(|ws| ws.into_iter().map(|w| w.label).collect())
        .unwrap_or_default()
}

pub fn workspace_exists(name: &str) -> Result<bool> {
    Ok(workspaces()?.iter().any(|w| w.label == name))
}

fn workspace_id(name: &str) -> Result<Option<String>> {
    Ok(workspaces()?.into_iter().find(|w| w.label == name).map(|w| w.id))
}

struct Workspace {
    id: String,
    label: String,
}

fn workspaces() -> Result<Vec<Workspace>> {
    let list = query(&["workspace", "list"])?;
    Ok(list
        .get("workspaces")
        .and_then(Value::as_array)
        .map(|ws| {
            ws.iter()
                .filter_map(|w| {
                    Some(Workspace {
                        id: w.get("workspace_id")?.as_str()?.to_string(),
                        label: w.get("label")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default())
}

/// Run a herdr subcommand for its side effect, checking only that it succeeded.
/// Side-effect commands (e.g. `pane run`) print nothing on success, so unlike
/// [`query`] this never tries to parse stdout.
fn run(args: &[&str]) -> Result<()> {
    output(args).map(|_| ())
}

/// Run a herdr subcommand and return its `result` payload as JSON.
fn query(args: &[&str]) -> Result<Value> {
    let stdout = output(args)?;
    let mut v: Value = serde_json::from_slice(&stdout)
        .with_context(|| format!("herdr {} returned invalid JSON", args.join(" ")))?;
    Ok(v["result"].take())
}

/// Run herdr, returning its stdout on success or an error carrying stderr.
fn output(args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("herdr")
        .args(args)
        .output()
        .with_context(|| format!("failed to run herdr {} (is it installed?)", args.join(" ")))?;

    if !out.status.success() {
        bail!(
            "herdr {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    Ok(out.stdout)
}

/// Pull a nested string field out of a herdr JSON payload.
fn string(v: &Value, path: &[&str]) -> Result<String> {
    let mut cur = v;
    for key in path {
        cur = cur
            .get(key)
            .with_context(|| format!("herdr response missing '{key}'"))?;
    }
    cur.as_str()
        .map(str::to_string)
        .with_context(|| format!("herdr field '{}' was not a string", path.join(".")))
}
