use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SKILL: &str = "gauntlet-loop";
const PLUGIN_ID: &str = "gauntlet-loop@gauntlet-loop";
const MARKETPLACE: &str = "robonuggets/gauntlet-loop";

const OWNED_FILES: &[&str] = &["SKILL.md"];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Scope {
    User,
    Project,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Scope::User => "user",
            Scope::Project => "project",
        }
    }

    fn parse(s: &str) -> Option<Scope> {
        match s {
            "user" => Some(Scope::User),
            "project" => Some(Scope::Project),
            _ => None,
        }
    }
}

fn fresh_install(scope: Scope) -> Result<(), String> {
    let cwd = match scope {
        Scope::Project => Some(std::env::current_dir().map_err(|e| e.to_string())?),
        Scope::User => None,
    };
    claude(
        &[
            OsStr::new("plugin"),
            OsStr::new("install"),
            OsStr::new(PLUGIN_ID),
            OsStr::new("--scope"),
            OsStr::new(scope.as_str()),
        ],
        cwd.as_deref(),
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Verdict {
    Managed,
    Unrecognised,
}

#[derive(Debug)]
struct Site {
    dir: PathBuf,
    /// None for the user-level skills directory.
    project: Option<PathBuf>,
    scope: Scope,
    verdict: Verdict,
    unexpected: Vec<String>,
    disabled: bool,
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Project roots, read from Claude Code's own index. Exact absolute paths.
fn project_roots(home: &Path) -> Vec<PathBuf> {
    let path = home.join(".claude.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        eprintln!("warning: could not parse {}", path.display());
        return Vec::new();
    };
    value
        .get("projects")
        .and_then(|p| p.as_object())
        .map(|o| o.keys().map(PathBuf::from).collect())
        .unwrap_or_default()
}

/// `skillOverrides` keys the bare skill name, so this never matches the
/// namespaced plugin skill.
fn is_disabled(root: &Path) -> bool {
    let path = root.join(".claude").join("settings.local.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| {
            v.get("skillOverrides")
                .and_then(|s| s.get(SKILL))
                .and_then(|s| s.as_str())
                .map(|s| s.eq_ignore_ascii_case("off"))
        })
        .unwrap_or(false)
}

fn classify(dir: &Path) -> (Verdict, Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return (Verdict::Unrecognised, vec!["<unreadable>".into()]);
    };
    let mut found: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != ".DS_Store")
        .collect();
    found.sort();

    let unexpected: Vec<String> = found
        .iter()
        .filter(|n| !OWNED_FILES.contains(&n.as_str()))
        .cloned()
        .collect();

    let has_all_owned = OWNED_FILES.iter().all(|o| found.iter().any(|f| f == o));

    if unexpected.is_empty() && has_all_owned {
        (Verdict::Managed, Vec::new())
    } else {
        (Verdict::Unrecognised, unexpected)
    }
}

fn discover(home: &Path) -> Vec<Site> {
    let mut sites = Vec::new();

    let user_dir = home.join(".claude").join("skills").join(SKILL);
    if user_dir.is_dir() {
        let (verdict, unexpected) = classify(&user_dir);
        sites.push(Site {
            dir: user_dir,
            project: None,
            scope: Scope::User,
            verdict,
            unexpected,
            disabled: is_disabled(home),
        });
    }

    for root in project_roots(home) {
        let dir = root.join(".claude").join("skills").join(SKILL);
        if dir.is_dir() {
            let (verdict, unexpected) = classify(&dir);
            sites.push(Site {
                disabled: is_disabled(&root),
                dir,
                project: Some(root),
                scope: Scope::Project,
                verdict,
                unexpected,
            });
        }
    }
    sites
}

fn guard(site: &Site, home: &Path) -> Result<(), String> {
    if !site.dir.ends_with(Path::new("skills").join(SKILL)) {
        return Err(format!(
            "refusing: unexpected path shape {}",
            site.dir.display()
        ));
    }
    let permitted = match &site.project {
        Some(root) => site.dir.starts_with(root),
        None => site.dir.starts_with(home.join(".claude")),
    };
    if !permitted {
        return Err(format!(
            "refusing: {} is outside its root",
            site.dir.display()
        ));
    }
    Ok(())
}

fn claude(args: &[&OsStr], cwd: Option<&Path>) -> Result<String, String> {
    let mut cmd = Command::new("claude");
    cmd.args(args).stdin(Stdio::null());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("could not run `claude`: {e}. Is Claude Code on your PATH?"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn plugin_installed(cwd: Option<&Path>) -> bool {
    let Ok(json) = claude(
        &[
            OsStr::new("plugin"),
            OsStr::new("list"),
            OsStr::new("--json"),
        ],
        cwd,
    ) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&json)
        .ok()
        .and_then(|v| {
            v.as_array().map(|a| {
                a.iter()
                    .any(|e| e.get("id").and_then(|i| i.as_str()) == Some(PLUGIN_ID))
            })
        })
        .unwrap_or(false)
}

fn clear_override(root: &Path) -> Result<bool, String> {
    let path = root.join(".claude").join("settings.local.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(false);
    };
    let mut value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;

    let removed = value
        .get_mut("skillOverrides")
        .and_then(|s| s.as_object_mut())
        .map(|o| o.remove(SKILL).is_some())
        .unwrap_or(false);

    if !removed {
        return Ok(false);
    }
    if value
        .get("skillOverrides")
        .and_then(|s| s.as_object())
        .is_some_and(|o| o.is_empty())
    {
        value.as_object_mut().map(|o| o.remove("skillOverrides"));
    }
    let mut out = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    out.push('\n');
    fs::write(&path, out).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

/// Fatal on failure: without it every install fails with a misleading
/// "not found in marketplace".
fn ensure_marketplace() -> Result<(), String> {
    match claude(
        &[
            OsStr::new("plugin"),
            OsStr::new("marketplace"),
            OsStr::new("add"),
            OsStr::new(MARKETPLACE),
        ],
        None,
    ) {
        Ok(_) => Ok(()),
        Err(e) if e.contains("already") => Ok(()),
        Err(e) => Err(format!(
            "could not register marketplace `{MARKETPLACE}`: {e}"
        )),
    }
}

fn migrate(site: &Site, home: &Path) -> Result<Vec<String>, String> {
    guard(site, home)?;
    let mut log = Vec::new();
    let cwd = site.project.as_deref();

    // Install before removing, so a failure leaves the existing skill working.
    claude(
        &[
            OsStr::new("plugin"),
            OsStr::new("install"),
            OsStr::new(PLUGIN_ID),
            OsStr::new("--scope"),
            OsStr::new(site.scope.as_str()),
        ],
        cwd,
    )?;
    if !plugin_installed(cwd) {
        return Err("plugin did not appear after install; nothing was removed".into());
    }
    log.push(format!(
        "installed {PLUGIN_ID} (scope: {})",
        site.scope.as_str()
    ));

    if site.disabled {
        claude(
            &[
                OsStr::new("plugin"),
                OsStr::new("disable"),
                OsStr::new(PLUGIN_ID),
                OsStr::new("--scope"),
                OsStr::new(site.scope.as_str()),
            ],
            cwd,
        )?;
        log.push("re-disabled to match previous state".into());
    }

    fs::remove_dir_all(&site.dir).map_err(|e| format!("{}: {e}", site.dir.display()))?;
    log.push(format!("removed {}", site.dir.display()));

    let root = site.project.clone().unwrap_or_else(|| home.to_path_buf());
    if clear_override(&root)? {
        log.push("cleared stale skillOverrides entry".into());
    }
    Ok(log)
}

fn print_report(sites: &[Site], write: bool) {
    if sites.is_empty() {
        println!("No hand-copied installs of `{SKILL}` found.");
        if !write {
            println!("Re-run with --write to install the plugin anyway.");
        }
        return;
    }
    let managed = sites
        .iter()
        .filter(|s| s.verdict == Verdict::Managed)
        .count();
    let skipped = sites.len() - managed;

    println!(
        "Found {} hand-copied install(s) of `{SKILL}`:\n",
        sites.len()
    );
    for s in sites {
        let where_ = match &s.project {
            Some(p) => p.display().to_string(),
            None => "(user level)".into(),
        };
        let state = if s.disabled { "disabled" } else { "enabled" };
        match s.verdict {
            Verdict::Managed => {
                println!("  [ok]   {where_}\n         {} ({state})", s.dir.display());
            }
            Verdict::Unrecognised => {
                println!(
                    "  [skip] {where_}\n         {} ({state})\n         contains files we did not ship: {}",
                    s.dir.display(),
                    s.unexpected.join(", ")
                );
            }
        }
    }
    println!();
    if skipped > 0 {
        println!(
            "{skipped} skipped. Those directories were modified locally, so they are left alone."
        );
        println!("Review them yourself, then delete them once you are satisfied.\n");
    }
    if !write {
        println!("This was a dry run. Nothing has changed.");
        println!(
            "Re-run with --write to replace the {managed} recognised install(s) with the plugin."
        );
    }
}

fn print_json(sites: &[Site]) {
    let mut out = Vec::new();
    for s in sites {
        let mut m = BTreeMap::new();
        m.insert("directory", serde_json::json!(s.dir.display().to_string()));
        m.insert(
            "project",
            match &s.project {
                Some(p) => serde_json::json!(p.display().to_string()),
                None => serde_json::Value::Null,
            },
        );
        m.insert("scope", serde_json::json!(s.scope.as_str()));
        m.insert(
            "verdict",
            serde_json::json!(match s.verdict {
                Verdict::Managed => "managed",
                Verdict::Unrecognised => "unrecognised",
            }),
        );
        m.insert("disabled", serde_json::json!(s.disabled));
        m.insert("unexpectedFiles", serde_json::json!(s.unexpected));
        out.push(m);
    }
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
}

const USAGE: &str = "\
gauntlet-loop-migrate — replace hand-copied installs of the gauntlet-loop skill
with the plugin, preserving each one's enabled/disabled state.

USAGE:
    gauntlet-loop-migrate [--write] [--json] [--scope <user|project>]

OPTIONS:
    --write           Perform the migration. Without this, nothing changes.
    --json            Emit findings as JSON and exit. Implies a dry run.
    --scope <scope>   Where to install when there is nothing to migrate.
                      Defaults to user. Ignored for sites we found, which are
                      installed at the scope their hand-copy occupied.
    --help            Show this message.

It reads Claude Code's own project index to find installs. It does not walk
your filesystem, and it never removes a directory containing anything other
than the file this project ships.
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{USAGE}");
        return;
    }
    let write = args.iter().any(|a| a == "--write");
    let as_json = args.iter().any(|a| a == "--json");

    let mut fallback_scope = Scope::User;
    let mut skip_next = false;
    for (i, a) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        match a.as_str() {
            "--write" | "--json" | "--help" | "-h" => {}
            "--scope" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("--scope needs a value (user or project)\n");
                    eprint!("{USAGE}");
                    std::process::exit(2);
                };
                let Some(s) = Scope::parse(v) else {
                    eprintln!("unknown scope: {v} (expected user or project)\n");
                    eprint!("{USAGE}");
                    std::process::exit(2);
                };
                fallback_scope = s;
                skip_next = true;
            }
            other => {
                eprintln!("unknown argument: {other}\n");
                eprint!("{USAGE}");
                std::process::exit(2);
            }
        }
    }

    let Some(home) = home() else {
        eprintln!("error: could not determine your home directory (HOME / USERPROFILE unset).");
        std::process::exit(1);
    };

    let sites = discover(&home);

    if as_json {
        print_json(&sites);
        return;
    }
    if !write {
        print_report(&sites, false);
        return;
    }

    print_report(&sites, true);

    if let Err(e) = ensure_marketplace() {
        eprintln!("\nerror: {e}");
        eprintln!("nothing was changed.");
        std::process::exit(1);
    }

    if sites.iter().all(|s| s.verdict != Verdict::Managed) {
        match fresh_install(fallback_scope) {
            Ok(()) => {
                println!(
                    "Installed {PLUGIN_ID} (scope: {}).",
                    fallback_scope.as_str()
                );
                println!("Invoke the skill as /gauntlet-loop:gauntlet-loop.");
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    let mut failures = 0;
    for site in sites.iter().filter(|s| s.verdict == Verdict::Managed) {
        let label = match &site.project {
            Some(p) => p.display().to_string(),
            None => "(user level)".into(),
        };
        println!("\n{label}");
        match migrate(site, &home) {
            Ok(log) => {
                for line in log {
                    println!("  - {line}");
                }
            }
            Err(e) => {
                failures += 1;
                eprintln!("  ! {e}");
                eprintln!("  ! left unchanged");
            }
        }
    }
    println!();
    if failures > 0 {
        eprintln!("{failures} site(s) failed and were left unchanged.");
        std::process::exit(1);
    }
    println!("Done. Invoke the skill as /gauntlet-loop:gauntlet-loop from now on.");
}
