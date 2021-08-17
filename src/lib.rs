use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub struct Exec {
    pub name: String,
    pub exec: String,
    pub terminal: bool,
}

// TODO: find out how to make it do something more intelligent with '%'s
fn process_exec(exec: &str) -> String {
    let res: Vec<&str> = exec
        .split(' ')
        .filter(|arg| (*arg).chars().next().unwrap() != '%')
        .collect();

    res.join(" ")
}

impl Exec {
    fn new(name: &str, exec: &str, terminal: bool) -> Self {
        Self {
            name: String::from(name),
            exec: String::from(exec),
            terminal,
        }
    }

    /// spawns the process with `exec` and provided `term_cmd` if
    /// the desktop entry contained `Terminal=true`
    pub fn run(&self, term_cmd: &str) {
        let exec = process_exec(&self.exec);

        let mut child = {
            if self.terminal {
                spawn(&format!("{} {}", term_cmd, &exec))
            } else {
                spawn(&exec)
            }
        }
        .expect(&format!("failed to spawn process '{}'", exec));

        child.wait().unwrap();
    }
}

/// runs `dmenu_cmd` and returns exec corresponding to dmenu's
/// output
pub fn run_dmenu<'a, I: std::iter::Iterator<Item = &'a Exec> + Clone>(
    execs: I,
    dmenu_cmd: &str,
) -> Option<Exec> {
    use std::io::Write;

    let names: Vec<String> = execs.clone().map(|exec| exec.name.clone()).collect();
    let names = names.join("\n");

    let mut dmenu = spawn(dmenu_cmd)?;

    dmenu
        .stdin
        .as_mut()
        .unwrap()
        .write_all(names.as_bytes())
        .ok()?;
    let output = dmenu.wait_with_output().ok()?;
    let output = String::from_utf8(output.stdout).ok()?;

    Some((execs.clone().find(|exec| exec.name == output.trim_end())?).clone())
}

use freedesktop_entry_parser::parse_entry;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

fn get_entries_from_path(dir: &Path) -> io::Result<Vec<DirEntry>> {
    let mut entries: Vec<DirEntry> = vec![];

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let mut sub_entries = get_entries_from_path(&path)?;
                entries.append(&mut sub_entries);
            } else {
                entries.push(entry);
            }
        }
    }
    Ok(entries)
}

pub fn get_execs(paths: &Vec<String>) -> Vec<Exec> {
    let execs: Vec<Exec> = paths
        .iter()
        .map(|path| get_entries_from_path(Path::new(path)).unwrap())
        .flatten()
        .filter_map(|direntry| {
            let ent = parse_entry(direntry.path()).ok()?;
            if let Some(nodisplay) = ent.section("Desktop Entry").attr("NoDisplay") {
                if nodisplay == "true" {
                    return None;
                }
            }

            let exec = Exec::new(
                ent.section("Desktop Entry").attr("Name")?,
                ent.section("Desktop Entry").attr("Exec")?,
                match ent.section("Desktop Entry").attr("Terminal") {
                    Some("true") => true,
                    _ => false,
                },
            );

            Some(exec)
        })
        .collect();

    execs
}

fn spawn(cmd: &str) -> Option<Child> {
    let mut args = cmd.split(' ');
    Command::new(args.next()?)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()
}

pub struct RdlConfig {
    paths: Vec<String>,
    dmenu: String,
    terminal: String,
    unique: bool,
}

impl RdlConfig {
    pub fn update_with_clap_matches(&mut self, matches: clap::ArgMatches) {
        if let Some(dmenu_cmd) = matches.value_of("dmenu") {
            self.dmenu = String::from(dmenu_cmd);
        }

        if let Some(term) = matches.value_of("term") {
            self.terminal = String::from(term);
        }

        if let Some(paths) = matches.value_of("paths") {
            self.paths = paths.split(":").map(|s| String::from(s)).collect();
        }

        if matches.is_present("unique") {
            self.unique = true;
        }
    }

    pub fn run(&self) {
        let execs = get_execs(&self.paths);

        use itertools::Itertools;
        let to_run = match self.unique {
            true => run_dmenu(execs.iter().unique_by(|a| a.name.clone()), &self.dmenu),
            false => run_dmenu(execs.iter(), &self.dmenu),
        };

        if let Some(to_run) = to_run {
            to_run.run(&self.terminal);
        }
    }

    pub fn update_with_config_file(&mut self, path: &std::path::Path) {
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(..) => return,
        };

        let doc = match yaml_rust::YamlLoader::load_from_str(&content) {
            Ok(loader) => loader[0].clone(),
            Err(..) => return,
        };

        if let Some(dmenu) = doc["dmenu"].as_str() {
            self.dmenu = String::from(dmenu);
        }

        if let Some(term) = doc["term"].as_str() {
            self.terminal = String::from(term);
        }

        if let Some(paths) = doc["paths"].as_vec() {
            self.paths = paths
                .iter()
                .map(|y| String::from(y.as_str().unwrap_or("")))
                .collect();
        }

        if let Some(unique) = doc["unique"].as_bool() {
            self.unique = unique;
        }
    }
}

impl Default for RdlConfig {
    fn default() -> Self {
        Self {
            paths: vec![
                format!(
                    "{}/.local/share/applications",
                    std::env::var("HOME").unwrap()
                ),
                String::from("/usr/share/applications"),
            ],
            dmenu: String::from("dmenu"),
            terminal: {
                let term = std::env::var("TERM").unwrap_or(String::from("xterm"));
                format!("{} -e", term)
            },
            unique: false,
        }
    }
}
