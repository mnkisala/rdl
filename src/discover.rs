use std::{fs::DirEntry, io::Read, path::Path};

use crate::Exec;

fn parse(path: impl AsRef<Path>) -> Option<Exec> {
    let mut file = std::fs::File::open(path).ok()?;

    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).ok()?;

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
                "Name" => name = Some(toks.next()?.to_owned()),
                "Exec" => exec = Some(toks.next()?.to_owned()),
                "Terminal" => terminal = Some(toks.next()?),
                "NoDisplay" => {
                    if toks.next()? == "true" {
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

fn get_entries_from_path(dir: &Path) -> std::io::Result<Vec<DirEntry>> {
    let mut entries: Vec<DirEntry> = vec![];

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let mut sub_entries = get_entries_from_path(&path)?;
                entries.append(&mut sub_entries);
            } else {
                if let Some(ext) = path.extension() {
                    use std::os::unix::ffi::OsStrExt;
                    if ext.as_bytes() == b"desktop" {
                        entries.push(entry);
                    }
                }
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
        .filter_map(|direntry| parse(direntry.path()))
        .collect();

    execs
}
