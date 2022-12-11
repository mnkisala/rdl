use std::process::{Child, Command, Stdio};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// ':' separated absolute paths to directories with desktop entries
    #[arg(short, long)]
    paths: Option<String>,
    /// dmenu command
    #[arg(short, long)]
    dmenu: Option<String>,
    /// terminal command to use when executing terminal applications
    #[arg(short, long)]
    term: Option<String>,
    /// deduplicates entries by name (first parsed stays)
    #[arg(short, long)]
    unique: Option<bool>,
}

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
    fn new(name: String, exec: String, terminal: bool) -> Self {
        Self {
            name,
            exec,
            terminal,
        }
    }

    /// spawns the process with `exec` and provided `term_cmd` if
    /// the desktop entry contained `Terminal=true`
    pub fn run(&self, term_cmd: &str) {
        let exec = process_exec(&self.exec);

        let mut child = {
            if self.terminal {
                println!("running: {} {}", term_cmd, &exec);
                spawn(&format!("{} {}", term_cmd, &exec))
            } else {
                println!("running: {}", &exec);
                spawn(&format!("{}", &exec))
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

use std::fs::{self, DirEntry};
use std::io::{self, Read};
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

fn parse(path: impl AsRef<Path>) -> Option<Exec> {
    let mut file = std::fs::File::open(path).ok()?;

    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    // SAFETY: desktop entries are requred to be encoded with utf-8
    // source: [https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s03.html]
    let content = unsafe { std::str::from_utf8_unchecked(&buf) };

    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut terminal: Option<&str> = None;

    for line in content.lines() {
        let mut toks = line.split('=');

        if let Some(start_tok) = toks.next() {
            match start_tok {
                "Name" => name = Some(toks.next().unwrap().to_owned()),
                "Exec" => exec = Some(toks.next().unwrap().to_owned()),
                "Terminal" => terminal = Some(toks.next().unwrap()),
                "NoDisplay" => {
                    if toks.next().unwrap() == "true" {
                        return None;
                    }
                }
                _ => continue,
            }
        }

        // early exit
        if name.is_some() && exec.is_some() && terminal.is_some() {
            break;
        }
    }

    Some(Exec::new(
        name?,
        exec?,
        match terminal {
            Some("true") => true,
            _ => false,
        },
    ))
}

pub fn get_execs(paths: &Vec<String>) -> Vec<Exec> {
    let execs: Vec<Exec> = paths
        .iter()
        .map(|path| get_entries_from_path(Path::new(path)).unwrap())
        .flatten()
        .filter_map(|direntry| parse(direntry.path()))
        .collect();

    execs
}

fn spawn(cmd: &str) -> Option<Child> {
    //let mut args = cmd.split(' ');
    Command::new("/bin/sh")
        .args(vec!["-c", cmd])
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
    pub fn update_with_args(&mut self, args: Args) {
        if let Some(dmenu_cmd) = args.dmenu {
            self.dmenu = String::from(dmenu_cmd);
        }

        if let Some(term) = args.term {
            self.terminal = String::from(term);
        }

        if let Some(paths) = args.paths {
            self.paths = paths.split(":").map(|s| String::from(s)).collect();
        }

        if let Some(unique) = args.unique {
            self.unique = unique;
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
