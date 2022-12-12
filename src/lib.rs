use std::process::{Child, Command, Stdio};

pub mod cli;
pub mod config;
pub mod discover;

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

fn spawn(cmd: &str) -> Option<Child> {
    //let mut args = cmd.split(' ');
    Command::new("/bin/sh")
        .args(vec!["-c", cmd])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()
}

pub fn run(config: &crate::config::RdlConfig) {
    let execs = crate::discover::get_execs(&config.paths);

    use itertools::Itertools;
    let to_run = match config.unique {
        true => run_dmenu(execs.iter().unique_by(|a| a.name.clone()), &config.dmenu),
        false => run_dmenu(execs.iter(), &config.dmenu),
    };

    if let Some(to_run) = to_run {
        to_run.run(&config.terminal);
    }
}
