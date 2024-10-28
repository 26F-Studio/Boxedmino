use crate::slint_types::ErrorWindow;
use slint::ComponentHandle;

pub fn open(
    title: Option<String>,
    message: Option<String>,
    details: Option<String>,
) -> Result<ErrorWindow, slint::PlatformError> {
    let error_window = ErrorWindow::new()?;
    error_window.set_error_title(
        title.unwrap_or("Error".to_string()).into()
    );
    error_window.set_error_message(
        message.unwrap_or("An error occurred.".to_string()).into()
    );
    error_window.set_error_details(
        details.unwrap_or("No details provided.".to_string()).into()
    );
    let weak = error_window.as_weak();
    error_window.on_dismiss(move || {
        weak.unwrap().window().hide().unwrap();
    });
    error_window.run()?;

    return Ok(error_window);
}

pub fn open_safe(
    title: Option<String>,
    message: Option<String>,
    details: Option<String>,
) {
    let result = open(
        title.clone(),
        message.clone(),
        details.clone()
    );
    if let Err(error) = result {
        eprintln!("Failed to open error window: {}", error);
        eprintln!("Title: {}", &title.unwrap_or("unspecified".into()).to_string());
        eprintln!("Message: {}", &message.unwrap_or("unspecified".into()).to_string());
        eprintln!("Details: {}", &details.unwrap_or("unspecified".into()).to_string());
    }
}