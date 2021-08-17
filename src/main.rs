use clap::{App, Arg};

fn main() {
    let matches = App::new("RDL Dmenu Launcher")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("paths")
                .short("p")
                .long("paths")
                .value_name("PATHS")
                .help(
                    r#"':' separated absolute paths to directories 
with desktop entries (default: '/usr/share/applications')"#,
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dmenu")
                .short("d")
                .long("dmenu")
                .value_name("COMMAND")
                .help("dmenu command for the launcher to run (default: 'dmenu')")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("term")
                .short("t")
                .long("term")
                .value_name("COMMAND")
                .help(
                    r#"terminal command to use when executing terminal 
applications (default: '$TERM -e', when $TERM 
is not set then falls back to xterm)"#,
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("unique")
                .long("unique")
                .short("u")
                .required(false)
                .help(r#"deduplicates entries by name allowing to make overrides, the 
first entry read stays, so if you've set paths to "$USER/.local/share/applications:/usr/share/applications"
it's going to override ones in /usr/share/applications with the ones from $USER/.local/share/applications"#)
                .takes_value(false),
        )
        .get_matches();

    let dmenu_cmd = matches.value_of("dmenu").unwrap_or("dmenu");
    let term_cmd: String = match matches.value_of("term") {
        Some(cmd) => String::from(cmd),
        None => {
            let term = std::env::var("TERM").unwrap_or(String::from("xterm"));
            format!("{} -e", term)
        }
    };

    let paths: Vec<&str> = matches
        .value_of("paths")
        .unwrap_or("/usr/share/applications")
        .split(":")
        .collect();

    let execs = rdl::get_execs(paths);

    use itertools::Itertools;
    let to_run = match matches.is_present("unique") {
        true => rdl::run_dmenu(execs.iter().unique_by(|a| a.name.clone()), dmenu_cmd),
        false => rdl::run_dmenu(execs.iter(), dmenu_cmd),
    };

    if let Some(to_run) = to_run {
        to_run.run(&term_cmd);
    }
}
