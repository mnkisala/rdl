use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// ':' separated absolute paths to directories with desktop entries
    #[arg(short, long)]
    pub paths: Option<String>,
    /// dmenu command
    #[arg(short, long)]
    pub dmenu: Option<String>,
    /// terminal command to use when executing terminal applications
    #[arg(short, long)]
    pub term: Option<String>,
    /// deduplicates entries by name (first parsed stays)
    #[arg(short, long)]
    pub unique: Option<bool>,
}
