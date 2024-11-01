use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::fs::{self, File};
use std::path::{PathBuf};
use std::collections::HashMap;
use std::thread;
use std::time::{Instant, Duration};
use std::sync::{mpsc, Arc, Mutex};
use reqwest::Url;
use slint::SharedString;
use slint::ComponentHandle;
use tokio::runtime::Runtime;
use crate::dirs::paths;
use crate::slint_types::ColdClearWaitWindow;
use zip::ZipArchive;

enum LoadingIPCMessage {
    AdvanceTo(
        /// The amount of bytes that have been downloaded.
        i32,
        /// The overall download rate in bytes per second
        i32,
        /// The ETA for the download
        SharedString
    ),
    SetTotal(i32),
    SetDeterminacy(bool),
    Finish,
    Error(reqwest::Error)
}

fn format_bytes(bytes: i32) -> SharedString {
    let bytes = bytes as f64;
    if bytes < 1e3 {
        return format!("{bytes:.0} bytes").into();
    } else if bytes < 1e6 {
        return format!("{:.2} KB", bytes / 1e3).into();
    } else if bytes < 1e9 {
        return format!("{:.2} MB", bytes / 1e6).into();
    } else {
        return format!("{:.2} GB", bytes / 1e9).into();
    }
}

#[test]
fn test_format_bytes() {
    let cases = [
        (0, "0 bytes"),
        (999, "999 bytes"),
        (1000, "1.00 KB"),
        (1024, "1.02 KB"),
        (1048575, "1.05 MB"),
        (1048576, "1.05 MB"),
        (1073741823, "1.07 GB"),
        (1073741824, "1.07 GB"),
        (2147483647, "2.15 GB"),
    ];

    for (input, expected) in cases {
        assert_eq!(format_bytes(input), expected);
    }
}

fn format_time(secs: i32) -> String {
    if secs < 60 {
        return format!("{secs:.0} seconds");
    } else if secs < 3600 {
        return format!("{:.0}:{:02.0}", secs / 60, secs % 60)
    } else {
        return format!(
            "{:.0}:{:02.0}:{:02.0}",
            secs / 3600,
            secs % 3600 / 60,
            secs % 60
        );
    }
}

#[test]
fn test_format_time() {
    let cases = [
        (0, "0 seconds"),
        (59, "59 seconds"),
        (60, "1:00"),
        (61, "1:01"),
        (3599, "59:59"),
        (3600, "1:00:00"),
        (3661, "1:01:01"),
        (86399, "23:59:59"),
        (86400, "24:00:00"),
    ];

    for (input, expected) in cases {
        assert_eq!(format_time(input), expected);
    }
}

pub fn download_cold_clear(version: &str) -> Result<(), reqwest::Error> {
    let (tx, rx) = mpsc::channel::<LoadingIPCMessage>();

    let window = ColdClearWaitWindow::new()
        .expect("Failed to open ColdClear loading window");

    window.on_format_bytes(format_bytes);

    let window_weak = window.as_weak();
    let version = version.to_owned();
    thread::spawn(move || {
        let rt = Runtime::new()
            .expect("Failed to create Tokio runtime");

        let version = version.as_str();
        let url = paths::get_cold_clear_download_url(version);
        let save_path = paths::get_cold_clear_download_path(version);

        rt.block_on(async move {
            let begin_time = Instant::now();

            let client = reqwest::Client::new();
            let response = client
                .get(url)
                .send()
                .await;

            if let Err(e) = response {
                tx.send(LoadingIPCMessage::Error(e))
                    .expect("Failed to send IPC message");
                return;
            }

            let mut response = response.unwrap();

            let total_size = response
                .content_length()
                .unwrap_or(0);

            let total_size = TryInto::<i32>::try_into(total_size)
                .expect("Failed to convert u64 to i32");

            tx.send(LoadingIPCMessage::SetTotal(total_size))
                .expect("Failed to send IPC message");

            tx.send(LoadingIPCMessage::SetDeterminacy(total_size != 0))
                .expect("Failed to send IPC message");

            let mut data = Vec::new();
            let mut downloaded_size = 0 as i32;

            let interrupted = Arc::new(Mutex::new(false));

            loop {
                let interrupted_clone = interrupted.clone();
                window_weak.upgrade_in_event_loop(move |window| {
                    let mut interrupted = interrupted_clone.lock().unwrap();

                    *interrupted = window.get_interrupted();
                }).expect("Failed to upgrade weak ref on interrupt check");

                if interrupted.lock().unwrap().clone() {
                    tx.send(LoadingIPCMessage::Finish)
                        .expect("Failed to send IPC message");
                    println!("ColdClear download interrupted!");
                    return;
                }
                
                // poll the `response` and get its progress
                let chunk = response.chunk().await;

                if let Err(e) = chunk {
                    tx.send(LoadingIPCMessage::Error(e))
                        .expect("Failed to send IPC message");
                    return;
                }

                let chunk = chunk.unwrap();

                if chunk.is_none() {
                    break;
                }

                let chunk = chunk.unwrap();

                downloaded_size += chunk.len() as i32;
                data.extend_from_slice(&chunk);

                let elapsed = begin_time.elapsed();

                let dl_rate = (
                    downloaded_size as f64 /
                    elapsed.as_secs_f64()
                ) as i32;
                
                let remaining_size = total_size - downloaded_size;
                let eta_secs = remaining_size / dl_rate;

                let advance_message = LoadingIPCMessage::AdvanceTo(
                    downloaded_size,
                    dl_rate,
                    format_time(eta_secs).into()
                );

                tx.send(advance_message)
                    .expect("Failed to send IPC message");

                thread::sleep(Duration::from_millis(200));
            }

            tx.send(LoadingIPCMessage::SetDeterminacy(false))
                .expect("Failed to send IPC message");

            fs::create_dir_all(save_path.parent().unwrap())
                .expect("Failed to create ColdClear download directory");

            let mut file = std::fs::File::create(save_path)
                .expect("Failed to create ColdClear download file");

            println!("Writing {} bytes to ColdClear path", data.len());

            file.write_all(data.as_ref())
                .expect("Failed to write ColdClear download file");

            tx.send(LoadingIPCMessage::Finish)
                .expect("Failed to send IPC message");
        });
    });

    let window_weak = window.as_weak();
    let window_thread = thread::spawn(move || {
        loop {
            let val = rx.recv();

            if let Err(_) = val {
                // Stream ended because transmitter no longer exists
                window_weak.upgrade_in_event_loop(|window| {
                    window.set_finished(true);
                    window.hide().expect("Failed to hide ColdClear loading window");
                }).expect("Error upgrading weak ref on event loop while breaking out");
                break;
            }
            
            let val = val.unwrap();

            match val {
                LoadingIPCMessage::AdvanceTo(bytes, rate, eta) => {
                    window_weak.upgrade_in_event_loop(move |window| {
                        window.set_bytes_done(bytes);
                        window.set_dl_rate(rate);
                        window.set_dl_eta(eta);
                    }).expect("Error upgrading weak ref on event loop while setting progress");
                }
                LoadingIPCMessage::SetTotal(bytes) => {
                    window_weak.upgrade_in_event_loop(move |window| {
                        window.set_bytes_total(bytes);
                    }).expect("Error upgrading weak ref on event loop while setting total");
                }
                LoadingIPCMessage::SetDeterminacy(determinate) => {
                    window_weak.upgrade_in_event_loop(move |window| {
                        window.set_indeterminate(!determinate)
                    }).expect("Error upgrading weak ref on event loop while setting determinacy");
                }
                LoadingIPCMessage::Finish => {
                    window_weak.upgrade_in_event_loop(|window| {
                        window.set_finished(true);
                        window.hide().expect("Failed to hide ColdClear loading window");
                    }).expect("Error upgrading weak ref on event loop while finishing");
                    break;
                }
                LoadingIPCMessage::Error(e) => {
                    return Err(e);
                }
            }
        }

        return Ok(());
    });

    let window_weak = window.as_weak();
    window.on_interrupt(move || {
        window_weak
            .unwrap().window().hide()
            .expect("Failed to hide ColdClear loading window");
    });

    window.run().expect("Failed to show ColdClear loading window");

    window.set_interrupted(true);

    return window_thread.join().expect("Failed to join window thread");
}

fn get_path_score(path: &str) -> i8 {
    #[cfg(target_arch = "x86_64")]
    {
        let pos_keywords = [
            "x64",
            "amd64",
            "x86_64"
        ];

        for keyword in pos_keywords.iter() {
            if path.contains(keyword) {
                return 1;
            }
        }

        let neg_keywords = [
            "x86",
            "i386",
            "i686"
        ];

        for keyword in neg_keywords.iter() {
            if path.contains(keyword) {
                return -1;
            }
        }

        return 0;
    }

    #[cfg(target_arch = "x86")]
    {
        let neg_keywords = [
            "x64",
            "amd64",
            "x86_64"
        ];

        for keyword in neg_keywords.iter() {
            if path.contains(keyword) {
                return -1;
            }
        }

        let pos_keywords = [
            "x86",
            "i386",
            "i686"
        ];

        for keyword in pos_keywords.iter() {
            if path.contains(keyword) {
                return 1;
            }
        }

        return 0;
    }

    #[cfg(target_arch = "aarch64")]
    {
        let pos_keywords = [
            "arm64",
            "aarch64"
        ];

        for keyword in pos_keywords.iter() {
            if path.contains(keyword) {
                return 1;
            }
        }

        let neg_keywords = [
            "arm",
            "armhf",
            "armv7"
        ];

        for keyword in neg_keywords.iter() {
            if path.contains(keyword) {
                return -1;
            }
        }

        return 0;
    }

    #[cfg(target_arch = "arm")]
    {
        let pos_keywords = [
            "armeabi-v7a",
            "armv7",
            "arm32"
        ];

        for keyword in pos_keywords.iter() {
            if path.contains(keyword) {
                return 1;
            }
        }

        let neg_keywords = [
            "arm64",
            "aarch64"
        ];

        for keyword in neg_keywords.iter() {
            if path.contains(keyword) {
                return -1;
            }
        }

        return 0;
    }

    #[allow(unreachable_code)]{
        return 0;
    }
}

fn pick_files_to_move(path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    // Traverse through the directories and pick files for flattenning
    // If identical filename, choose one with higher path score
    let path = path.to_path_buf();

    fn traverse(path: PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
        let entries = path.as_path().read_dir()?;

        let mut file_list: Vec<PathBuf> = Vec::new();

        for entry in entries {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                file_list.append(&mut traverse(entry.path())?);
                continue;
            }

            file_list.push(entry.path());
        }

        return Ok(file_list);
    }

    let file_list = traverse(path)?;

    let mut file_map: HashMap<OsString, PathBuf> = HashMap::new();

    for path in file_list {
        let name = path.file_name();

        if name.is_none() {
            return Err(
                format!("Failed to get filename for file: {:#?}", path).into()
            );
        }

        let name = name.unwrap();

        if file_map.contains_key(name) {
            let other_path = file_map.get(name).unwrap();

            let path_str = path.to_str();
            let other_path_str = other_path.to_str();

            if path_str.is_none() {
                return Err(
                    format!("Failed to get UTF-8 path string for path: {:#?}", path).into()
                );
            }

            if other_path_str.is_none() {
                return Err(
                    format!("Failed to get UTF-8 path string for path: {:#?}", path).into()
                );
            }

            let cur_score = get_path_score(path_str.unwrap());
            let other_score = get_path_score(other_path_str.unwrap());

            if cur_score > other_score {
                file_map.insert(name.into(), path.to_owned());
            }
        } else {
            file_map.insert(name.into(), path.to_owned());
        }
    }

    return Ok(
        file_map
            .values()
            .cloned()
            .collect()
    );
}

pub fn unpack_cold_clear(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let zip_path = paths::get_cold_clear_download_path(version);
    let zip_path = zip_path.as_path();

    if !zip_path.exists() {
        download_cold_clear(version)?;
    }

    let zip_file = std::fs::File::open(zip_path)?;

    let mut zip_archive = ZipArchive::new(&zip_file);

    if let Err(_) = zip_archive {
        eprintln!("ColdClear zip archive at '{zip_path:#?}' seems to be invalid. Redownloading.");
        download_cold_clear(version)?;

        zip_archive = ZipArchive::new(&zip_file);
    }

    let mut zip_archive = zip_archive?;

    let lib_path = paths::get_sandboxed_save_path().join("lib");
    let temp_lib_path = paths::get_sandboxed_save_path().join("~lib");

    zip_archive.extract(&temp_lib_path)?;

    let files_to_move = pick_files_to_move(&temp_lib_path)?;

    for path in files_to_move {
        let dest = lib_path.join(path.file_name().unwrap());
        fs::rename(path, dest)?;
    }

    fs::remove_dir_all(temp_lib_path)?;
    
    return Ok(());
}

pub fn get_available_offline_versions() -> Result<Vec<String>, std::io::Error> {
    let path = paths::get_cold_clear_download_path("")
        .parent()?;

    let entries = fs::read_dir(path)?;

    let mut versions: Vec<String> = Vec::new();

    for entry in entries {
        let entry = entry?;

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = path.file_name();
        if name.is_none() {
            continue;
        }
        let name = name.unwrap()
            .to_str()
            .expect("Failed to get UTF-8 string from OsStr");

        if !name.ends_with(".zip") {
            continue;
        }

        let version = name
            .replace(".zip", "");

        versions.push(version);
    }

    return Ok(versions);
}

pub fn get_available_versions() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let api_url = paths::COLD_CLEAR_RELEASES_API_URL;

    let response = reqwest::blocking::get(api_url)?;

    let json = response.text()?;

    let json: serde_json::Value = serde_json::from_str(&json)?;

    let releases = match json {
        serde_json::Value::Array(arr) => arr,
        _ => return Err("Expected JSON array from GitHub API".into())
    };

    let mut versions: Vec<String> = Vec::new();

    for release in releases {
        let version = match release["tag_name"].as_str() {
            Some(version) => version,
            None => continue
        };

        versions.push(version.to_owned());
    }

    return Ok(versions);
}