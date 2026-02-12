use gtk4::prelude::*;
use libadwaita::prelude::*;

/// Handles returned from building the dashboard window.
pub struct DashboardWidgets {
    pub window: libadwaita::ApplicationWindow,
    pub status_label: gtk4::Label,
    pub words_label: gtk4::Label,
    pub prompts_label: gtk4::Label,
    pub hotkey_label: gtk4::Label,
    pub change_hotkey_button: gtk4::Button,
    pub api_key_row: libadwaita::PasswordEntryRow,
    pub progress_bar: gtk4::ProgressBar,
    pub prompts_row: libadwaita::ActionRow,
}

/// Build the main dashboard window.
pub fn build_dashboard(
    app: &libadwaita::Application,
    initial_status: &str,
    initial_words: usize,
    initial_prompts: usize,
    initial_hotkey: &str,
    initial_api_key: &str,
) -> DashboardWidgets {
    let window = libadwaita::ApplicationWindow::builder()
        .application(app)
        .title("Voice Prompt")
        .default_width(450)
        .default_height(500)
        .build();

    let toolbar_view = libadwaita::ToolbarView::new();
    let header = libadwaita::HeaderBar::new();

    // Add menu button
    let menu_button = gtk4::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");

    let menu = gtk4::gio::Menu::new();
    menu.append(Some("About Voice Prompt"), Some("app.about"));
    menu.append(Some("Hide Window"), Some("app.hide-window"));
    menu.append(Some("Quit"), Some("app.quit"));

    menu_button.set_menu_model(Some(&menu));
    header.pack_end(&menu_button);

    toolbar_view.add_top_bar(&header);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    // --- Status group ---
    let status_group = libadwaita::PreferencesGroup::new();
    status_group.set_title("Status");

    let status_row = libadwaita::ActionRow::builder()
        .title("Current State")
        .build();
    let status_label = gtk4::Label::new(Some(initial_status));
    status_label.add_css_class("dim-label");
    status_row.add_suffix(&status_label);
    status_group.add(&status_row);

    content.append(&status_group);
    content.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    // --- Statistics group ---
    let stats_group = libadwaita::PreferencesGroup::new();
    stats_group.set_title("Statistics");
    stats_group.set_margin_top(12);

    let words_row = libadwaita::ActionRow::builder()
        .title("Total Words Generated")
        .build();
    let words_label = gtk4::Label::new(Some(&initial_words.to_string()));
    words_label.add_css_class("dim-label");
    words_row.add_suffix(&words_label);
    stats_group.add(&words_row);

    let prompts_row = libadwaita::ActionRow::builder()
        .title("Total Prompts")
        .activatable(true)
        .build();
    let prompts_label = gtk4::Label::new(Some(&initial_prompts.to_string()));
    prompts_label.add_css_class("dim-label");
    prompts_row.add_suffix(&prompts_label);
    let chevron = gtk4::Image::from_icon_name("go-next-symbolic");
    prompts_row.add_suffix(&chevron);
    stats_group.add(&prompts_row);

    content.append(&stats_group);
    content.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    // --- Hotkey group ---
    let hotkey_group = libadwaita::PreferencesGroup::new();
    hotkey_group.set_title("Hotkey");
    hotkey_group.set_margin_top(12);

    let hotkey_row = libadwaita::ActionRow::builder()
        .title("Record Toggle")
        .build();
    let hotkey_label = gtk4::Label::new(Some(initial_hotkey));
    hotkey_label.add_css_class("dim-label");
    hotkey_row.add_suffix(&hotkey_label);

    let change_hotkey_button = gtk4::Button::builder()
        .label("Change")
        .valign(gtk4::Align::Center)
        .build();
    hotkey_row.add_suffix(&change_hotkey_button);
    hotkey_group.add(&hotkey_row);

    content.append(&hotkey_group);
    content.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    // --- API Key group ---
    let api_group = libadwaita::PreferencesGroup::new();
    api_group.set_title("Gemini API");
    api_group.set_margin_top(12);

    let api_key_row = libadwaita::PasswordEntryRow::builder()
        .title("API Key")
        .text(initial_api_key)
        .build();
    api_group.add(&api_key_row);

    content.append(&api_group);

    // --- Download progress bar ---
    let progress_bar = gtk4::ProgressBar::new();
    progress_bar.set_margin_top(16);
    progress_bar.set_visible(false);
    progress_bar.set_show_text(true);
    progress_bar.set_text(Some("Downloading whisper model..."));
    content.append(&progress_bar);

    // Assemble
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&content)
        .build();
    toolbar_view.set_content(Some(&scrolled));
    window.set_content(Some(&toolbar_view));

    DashboardWidgets {
        window,
        status_label,
        words_label,
        prompts_label,
        hotkey_label,
        change_hotkey_button,
        api_key_row,
        progress_bar,
        prompts_row,
    }
}
