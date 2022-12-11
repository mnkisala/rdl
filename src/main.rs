use rdl::Args;

fn main() {
    let args = <Args as clap::Parser>::parse();

    let mut config = rdl::RdlConfig::default();

    config.update_with_config_file(std::path::Path::new(&format!(
        "{}/.config/rdl.yaml",
        std::env::var("HOME").unwrap()
    )));

    config.update_with_config_file(std::path::Path::new(&format!(
        "{}/.config/rdl/config.yaml",
        std::env::var("HOME").unwrap()
    )));

    config.update_with_args(args);

    config.run();
}
