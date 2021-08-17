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
with desktop entries (default: '(path to home)/.local/share/applications:/usr/share/applications')"#,
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
first entry read stays, so if you've set paths to "(path to your home)/.local/share/applications:/usr/share/applications"
it's going to override ones in /usr/share/applications with the ones from (path to your home)/.local/share/applications"#)
                .takes_value(false),
        )
        .get_matches();

    let mut config = rdl::RdlConfig::default();

    config.update_with_config_file(std::path::Path::new(&format!(
        "{}/.config/rdl.yaml",
        std::env::var("HOME").unwrap()
    )));

    config.update_with_config_file(std::path::Path::new(&format!(
        "{}/.config/rdl/config.yaml",
        std::env::var("HOME").unwrap()
    )));

    config.update_with_clap_matches(matches);

    config.run();
}
