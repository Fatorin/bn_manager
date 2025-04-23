use crate::bot::ResponseCode;
use crate::model::map::MapInfo;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, io};

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

pub fn verify_user_credentials(
    folder: &str,
    username: &str,
    password: &str,
) -> Result<(), ResponseCode> {
    let file_name = match check_exist(folder, username) {
        Ok(_) => {
            return Err(ResponseCode::NotRegistered);
        }
        Err(err) => match err {
            ResponseCode::UserIdTaken(username) => username,
            _ => {
                return Err(ResponseCode::ServerError);
            }
        },
    };

    let file_path = Path::new(folder).join(file_name);
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(err) => {
            println!("讀取檔案失敗: {}", err);
            return Err(ResponseCode::ServerError);
        }
    };

    let pattern = Regex::new(r#""BNET\\\\acct\\\\passhash1"="([^"]*)""#).unwrap();
    let captures = match pattern.captures(&content) {
        Some(captures) => captures,
        None => {
            println!("檔案中未找到密碼格式");
            return Err(ResponseCode::ServerError);
        }
    };

    let stored_password = captures.get(1).map_or("", |m| m.as_str());
    if stored_password.is_empty() {
        println!("檔案出現空密碼");
        return Err(ResponseCode::ServerError);
    };

    if stored_password == password {
        Ok(())
    } else {
        Err(ResponseCode::InvalidPasswordInput)
    }
}

pub fn check_exist(folder: &str, new_file_name: &str) -> Result<usize, ResponseCode> {
    let mut file_count: usize = 0;

    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(_) => {
            println!("can't load folder");
            return Err(ResponseCode::ServerError);
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
            Err(err) => {
                println!("can't get file metadata, error: {}", err);
                continue;
            }
        };

        if metadata.is_file() {
            file_count += 1;
            let file_name = entry.file_name();
            if file_name.to_string_lossy().to_lowercase() == new_file_name.to_lowercase() {
                return Err(ResponseCode::UserIdTaken(
                    file_name.to_string_lossy().to_string(),
                ));
            }
        }
    }

    Ok(file_count)
}
