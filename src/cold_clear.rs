use std::io::Write;
use std::thread;
use std::time::{Instant, Duration};
use std::sync::{mpsc, Arc, Mutex};
use slint::SharedString;
use slint::ComponentHandle;
use tokio::runtime::Runtime;
use crate::dirs::paths;
use crate::slint_types::ColdClearWaitWindow;

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

fn format_time(secs: i32) -> String {
    if secs < 60 {
        return format!("{secs:.0} seconds");
    } else if secs < 3600 {
        return format!("{:.0}:{:2.0}", secs / 60, secs % 60)
    } else {
        return format!(
            "{:.0}:{:2.0}:{:2.0}",
            secs / 3600,
            secs % 3600 / 60,
            secs % 60
        );
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

            let bytes = response.bytes().await;

            if let Err(e) = bytes {
                tx.send(LoadingIPCMessage::Error(e))
                    .expect("Failed to send IPC message");
                return;
            }

            let bytes = bytes.unwrap();

            let mut file = std::fs::File::create(save_path)
                .expect("Failed to create ColdClear download file");

            file.write_all(bytes.as_ref())
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