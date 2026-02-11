use gtk4::prelude::*;
use libadwaita::prelude::*;

use crate::stats::PromptRecord;

/// Show a window listing past prompt history.
pub fn show_history_window(
    parent: &impl IsA<gtk4::Window>,
    history: &[PromptRecord],
) {
    let window = libadwaita::Window::builder()
        .title("Prompt History")
        .default_width(500)
        .default_height(550)
        .transient_for(parent)
        .modal(true)
        .build();

    let toast_overlay = libadwaita::ToastOverlay::new();

    let toolbar_view = libadwaita::ToolbarView::new();
    let header = libadwaita::HeaderBar::new();

    // Back button in header
    let back_btn = gtk4::Button::from_icon_name("go-previous-symbolic");
    back_btn.set_tooltip_text(Some("Back to main"));
    let win_for_back = window.clone();
    back_btn.connect_clicked(move |_| {
        win_for_back.close();
    });
    header.pack_start(&back_btn);

    toolbar_view.add_top_bar(&header);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    if history.is_empty() {
        let empty_label = gtk4::Label::new(Some("No prompts recorded yet."));
        empty_label.add_css_class("dim-label");
        empty_label.set_vexpand(true);
        empty_label.set_valign(gtk4::Align::Center);
        content.append(&empty_label);
    } else {
        let group = libadwaita::PreferencesGroup::new();
        group.set_title("Recent Prompts");

        for record in history.iter().rev() {
            let row = build_prompt_row(record, &toast_overlay);
            group.add(&row);
        }

        content.append(&group);
    }

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&content)
        .build();
    toolbar_view.set_content(Some(&scrolled));
    toast_overlay.set_child(Some(&toolbar_view));
    window.set_content(Some(&toast_overlay));
    window.present();
}

/// Build an ExpanderRow for a single prompt record.
fn build_prompt_row(
    record: &PromptRecord,
    toast_overlay: &libadwaita::ToastOverlay,
) -> libadwaita::ExpanderRow {
    let row = libadwaita::ExpanderRow::builder()
        .title(&record.timestamp)
        .build();

    // Truncated subtitle preview
    let preview: String = if record.text.len() > 100 {
        format!("{}...", &record.text[..100])
    } else {
        record.text.clone()
    };
    row.set_subtitle(&preview);

    // Word count suffix
    let count_label = gtk4::Label::new(
        Some(&format!("{} words", record.word_count)),
    );
    count_label.add_css_class("dim-label");
    row.add_suffix(&count_label);

    // Copy button suffix
    let copy_btn = gtk4::Button::from_icon_name("edit-copy-symbolic");
    copy_btn.set_valign(gtk4::Align::Center);
    copy_btn.set_tooltip_text(Some("Copy to clipboard"));

    let text_for_copy = record.text.clone();
    let toast_for_copy = toast_overlay.clone();
    copy_btn.connect_clicked(move |_| {
        let _ = crate::clipboard::copy_to_clipboard(&text_for_copy);
        let toast = libadwaita::Toast::new("Prompt copied to clipboard");
        toast.set_timeout(2);
        toast_for_copy.add_toast(toast);
    });
    row.add_suffix(&copy_btn);

    // Full text child row (visible when expanded)
    let full_text_row = libadwaita::ActionRow::new();
    let label = gtk4::Label::new(Some(&record.text));
    label.set_wrap(true);
    label.set_xalign(0.0);
    label.set_margin_top(4);
    label.set_margin_bottom(4);
    label.set_margin_start(8);
    label.set_margin_end(8);
    label.set_selectable(true);
    full_text_row.set_child(Some(&label));
    row.add_row(&full_text_row);

    row
}
