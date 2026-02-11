use gtk4::prelude::*;
use libadwaita::prelude::*;

use crate::config::HotkeyConfig;

/// Show a modal dialog that captures a new hotkey combo via evdev.
/// Calls `on_result` on the GTK main thread with the captured config.
pub fn show_hotkey_dialog<F>(parent: &libadwaita::ApplicationWindow, on_result: F)
where
    F: Fn(Option<HotkeyConfig>) + 'static,
{
    let dialog = libadwaita::AlertDialog::builder()
        .heading("Change Hotkey")
        .body("Press the desired key combination...\n(modifier + key, e.g. Ctrl+Space)\n\nTimes out after 10 seconds.")
        .build();
    dialog.add_response("cancel", "Cancel");

    // Spawn evdev capture in a background thread
    let (tx, rx) = async_channel::bounded::<Option<HotkeyConfig>>(1);

    std::thread::Builder::new()
        .name("hotkey-capture".into())
        .spawn(move || {
            let result = crate::hotkey::capture_hotkey_combo();
            let _ = tx.try_send(result);
        })
        .expect("Failed to spawn capture thread");

    // When the capture thread finishes, close dialog and deliver result
    let dialog_ref = dialog.clone();
    gtk4::glib::spawn_future_local(async move {
        if let Ok(result) = rx.recv().await {
            dialog_ref.close();
            on_result(result);
        }
    });

    // Show the dialog. Callback receives the response ID as GString.
    let parent_widget: Option<&gtk4::Widget> = Some(parent.upcast_ref());
    dialog.choose(parent_widget, None::<&gtk4::gio::Cancellable>, |_response_id| {});
}
