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

pub fn get_execs(paths: Vec<&str>) -> Vec<Exec> {
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
