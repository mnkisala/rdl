pub struct RdlConfig {
    pub paths: Vec<String>,
    pub dmenu: String,
    pub terminal: String,
    pub unique: bool,
}

impl RdlConfig {
    pub fn update_with_args(&mut self, args: crate::cli::Args) {
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
