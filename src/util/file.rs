use crate::model::map::MapInfo;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use regex::Regex;
use crate::settings::CONFIG;

pub fn read_files_in_directory(dir: &Path) -> io::Result<HashMap<String, MapInfo>> {
    let mut maps: HashMap<String, MapInfo> = HashMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match analysis_w3x(&path) {
                Ok(map_info) => {
                    maps.insert(map_info.name.clone(), map_info);
                }
                Err(e) => println!("Failed to read {}: {}", path.display(), e),
            }
        }
    }

    Ok(maps)
}

pub fn analysis_w3x_name(name: String, bytes: &[u8]) -> io::Result<MapInfo> {
    let new_buffer = &bytes[8..];

    let empty_byte: u8 = 0x00;
    let pos = new_buffer
        .iter()
        .position(|&x| x == empty_byte)
        .unwrap_or(new_buffer.len());
    let mut map_name = String::from_utf8_lossy(&new_buffer[..pos]).to_string();
    map_name = map_name.replace("|r", "");

    while let Some(pos) = map_name.find("|c") {
        if pos + 10 <= map_name.len() {
            map_name.replace_range(pos..pos + 10, "");
        } else {
            break;
        }
    }

    Ok(MapInfo { name, map_name })
}

fn analysis_w3x(path: &PathBuf) -> io::Result<MapInfo> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0; 128];
    file.read_exact(&mut buffer)?;

    let file_name = path
        .file_name()
        .expect("can't convert file name")
        .to_string_lossy();
    let map_info = analysis_w3x_name(file_name.to_string(), &buffer)?;

    Ok(map_info)
}

pub fn change_password(file_name: &str, pwd_hash: &str) -> io::Result<()> {
    let pattern = Regex::new(r#""BNET\\\\acct\\\\passhash1"="[^"]*""#).unwrap();
    let file_path = Path::new(&CONFIG.user_data_path).join(file_name);
    let mut content = String::new();
    let mut file = File::open(&file_path)?;
    file.read_to_string(&mut content)?;

    if pattern.is_match(&content) {
        let new_content = pattern.replace_all(&content, &format!(r#""BNET\\acct\\passhash1"="{}""#, pwd_hash));
        let mut file = File::create(&file_path)?;
        file.write_all(new_content.as_bytes())?;

        println!("change password succeed, username:{}", file_path.display());
    } else {
        println!("didn't found passowrd field, username:{}", file_path.display());
    }

    Ok(())
}

pub fn check_exist(folder: &str, new_file_name: &str) -> (usize, bool) {
    let mut file_count: usize = 0;

    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(_) => {
            println!("can't load folder");
            return (0, false);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                println!("can't load file");
                continue;
            }
        };

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                println!("can't get file metadata");
                continue;
            }
        };

        if metadata.is_file() {
            file_count += 1;
            let file_name = entry.file_name();
            if file_name.to_string_lossy().to_lowercase() == new_file_name.to_lowercase() {
                return (0, false);
            }
        }
    }

    (file_count, true)
}
