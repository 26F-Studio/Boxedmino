use std::io::Write;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use crate::consts::paths;

slint::include_modules!();

#[derive(Debug)]
enum LoadingIPCMessage {
    AdvanceTo(f32),
    SetDeterminacy(bool),
    Finish,
    Error(reqwest::Error)
}

pub fn download_cold_clear() -> Result<(), reqwest::Error> {
    let (tx, rx) = mpsc::channel::<LoadingIPCMessage>();

    let window = ColdClearWaitWindow::new()
        .expect("Failed to open ColdClear loading window");

    let dl_thread = thread::spawn(move || {
        async move {
            println!("CC DL Thread Started"); // DEBUG

            let url = paths::COLD_CLEAR_DOWNLOAD_URL;
            let save_path = paths::get_cold_clear_download_path();

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

            tx.send(LoadingIPCMessage::SetDeterminacy(total_size != 0))
                .expect("Failed to send IPC message");

            let mut downloaded_size = 0;

            loop {
                // poll the `response` and get its progress percentage
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

                downloaded_size += chunk.len() as u64;

                let progress = downloaded_size as f32 / total_size as f32;

                tx.send(LoadingIPCMessage::AdvanceTo(progress))
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

            println!("CC DL Thread Finished"); // DEBUG
        }
    });

    let window_weak = window.as_weak();
    let window_thread = thread::spawn(move || {
        loop {
            let val = rx.recv().expect("Failed to receive IPC message");

            println!("{:?}", val);

            match val {
                LoadingIPCMessage::AdvanceTo(progress) => {
                    // window.set_progress(progress);
                    window_weak.upgrade_in_event_loop(move |window| {
                        window.set_progress(progress);
                    }).expect("Error upgrading weak ref on event loop while setting progress");
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

    window.run().expect("Failed to show ColdClear loading window");

    return Ok(());
}