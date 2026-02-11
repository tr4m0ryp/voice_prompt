use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{self, Align};
#[cfg(target_os = "linux")]
use gtk4_layer_shell::LayerShell;

use crate::app::{BackendEvent, OverlayPhase};

const NUM_BARS: usize = 24;

/// Handles returned from building the overlay window.
pub struct OverlayWidgets {
    pub window: gtk4::Window,
    pub timer_label: gtk4::Label,
    pub waveform: gtk4::DrawingArea,
    pub audio_levels: Rc<RefCell<VecDeque<f32>>>,
    // Phase-transition widgets
    pub dot: gtk4::Label,
    pub recording_label: gtk4::Label,
    pub hbox: gtk4::Box,
    pub status_label: gtk4::Label,
}

/// Update overlay widgets to reflect the current pipeline phase.
pub fn set_overlay_phase(overlay: &OverlayWidgets, phase: &OverlayPhase) {
    match phase {
        OverlayPhase::Recording => {
            overlay.dot.set_visible(true);
            overlay.recording_label.set_visible(true);
            overlay.waveform.set_visible(true);
            overlay.timer_label.set_visible(true);
            overlay.status_label.set_visible(false);
            overlay.hbox.remove_css_class("done-bar");
        }
        OverlayPhase::Transcribing => {
            overlay.dot.set_visible(false);
            overlay.recording_label.set_visible(false);
            overlay.waveform.set_visible(false);
            overlay.timer_label.set_visible(false);
            overlay.status_label.set_text("Transcribing\u{2026}");
            overlay.status_label.set_visible(true);
            overlay.hbox.remove_css_class("done-bar");
        }
        OverlayPhase::Refining => {
            overlay.dot.set_visible(false);
            overlay.recording_label.set_visible(false);
            overlay.waveform.set_visible(false);
            overlay.timer_label.set_visible(false);
            overlay.status_label.set_text("Refining\u{2026}");
            overlay.status_label.set_visible(true);
            overlay.hbox.remove_css_class("done-bar");
        }
        OverlayPhase::Done(_) => {
            overlay.dot.set_visible(false);
            overlay.recording_label.set_visible(false);
            overlay.waveform.set_visible(false);
            overlay.timer_label.set_visible(false);
            overlay.status_label.set_text("Done \u{2713}");
            overlay.status_label.set_visible(true);
            overlay.hbox.add_css_class("done-bar");
        }
    }
}

/// Build the recording overlay bar.
pub fn build_overlay(
    app: &libadwaita::Application,
    backend_sender: async_channel::Sender<BackendEvent>,
) -> OverlayWidgets {
    let window = gtk4::Window::builder()
        .application(app)
        .title("Recording")
        .decorated(false)
        .resizable(false)
        .default_width(320)
        .default_height(44)
        .build();

    window.add_css_class("recording-overlay");

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_string(
        r#"
        window.recording-overlay {
            background-color: transparent;
        }
        .recording-bar {
            background-color: rgba(30, 30, 30, 0.90);
            border-radius: 22px;
            padding: 8px 20px;
        }
        .recording-bar.done-bar {
            background-color: rgba(30, 100, 30, 0.90);
        }
        .recording-dot {
            color: #ff3b30;
            font-size: 18px;
        }
        .recording-label {
            color: white;
            font-weight: bold;
            font-size: 14px;
        }
        .recording-timer {
            color: rgba(255, 255, 255, 0.7);
            font-size: 14px;
            font-family: monospace;
        }
        .overlay-status {
            color: white;
            font-weight: bold;
            font-size: 14px;
        }
        "#,
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    hbox.set_halign(Align::Center);
    hbox.set_valign(Align::Center);
    hbox.add_css_class("recording-bar");

    let dot = gtk4::Label::new(Some("\u{25CF}")); // ●
    dot.add_css_class("recording-dot");

    let recording_label = gtk4::Label::new(Some("Recording"));
    recording_label.add_css_class("recording-label");

    let audio_levels: Rc<RefCell<VecDeque<f32>>> =
        Rc::new(RefCell::new(VecDeque::from(vec![0.0; NUM_BARS])));
    let waveform = gtk4::DrawingArea::new();
    waveform.set_content_width(((3 + 2) * NUM_BARS) as i32);
    waveform.set_content_height(28);

    let levels_for_draw = audio_levels.clone();
    waveform.set_draw_func(move |_area, cr, width, height| {
        draw_waveform(cr, width, height, &levels_for_draw.borrow());
    });

    let timer_label = gtk4::Label::new(Some("00:00"));
    timer_label.add_css_class("recording-timer");

    let status_label = gtk4::Label::new(None);
    status_label.add_css_class("overlay-status");
    status_label.set_visible(false);

    hbox.append(&dot);
    hbox.append(&recording_label);
    hbox.append(&waveform);
    hbox.append(&timer_label);
    hbox.append(&status_label);

    window.set_child(Some(&hbox));

    // Click gesture to dismiss / re-copy on Done
    let click = gtk4::GestureClick::new();
    let sender_for_click = backend_sender;
    click.connect_released(move |_, _, _, _| {
        let _ = sender_for_click.try_send(BackendEvent::OverlayClicked);
    });
    window.add_controller(click);

    // Platform-specific window positioning
    #[cfg(target_os = "linux")]
    {
        let is_wayland = std::env::var("XDG_SESSION_TYPE")
            .map(|s| s == "wayland")
            .unwrap_or(false);

        if is_wayland && gtk4_layer_shell::is_supported() {
            window.init_layer_shell();
            window.set_layer(gtk4_layer_shell::Layer::Overlay);
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            window.set_margin(gtk4_layer_shell::Edge::Bottom, 30);
            window.set_anchor(gtk4_layer_shell::Edge::Left, false);
            window.set_anchor(gtk4_layer_shell::Edge::Right, false);
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use a floating undecorated window at bottom of screen.
        // GTK4 on macOS handles native window management.
        window.set_decorated(false);
    }

    window.set_visible(false);

    window.connect_close_request(|w| {
        w.set_visible(false);
        gtk4::glib::Propagation::Stop
    });

    OverlayWidgets {
        window,
        timer_label,
        waveform,
        audio_levels,
        dot,
        recording_label,
        hbox,
        status_label,
    }
}

fn draw_waveform(
    cr: &gtk4::cairo::Context,
    width: i32,
    height: i32,
    levels: &VecDeque<f32>,
) {
    let h = height as f64;
    let bar_w = 3.0;
    let gap = 2.0;
    let total_w = (bar_w + gap) * NUM_BARS as f64 - gap;
    let x_offset = (width as f64 - total_w) / 2.0;

    for (i, &level) in levels.iter().enumerate() {
        let clamped = (level as f64).clamp(0.0, 1.0);
        let bar_h = (2.0 + clamped * (h - 4.0)).max(2.0);
        let x = x_offset + i as f64 * (bar_w + gap);
        let y = (h - bar_h) / 2.0;
        let alpha = 0.3 + 0.7 * clamped;
        cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
        let _ = cr.rectangle(x, y, bar_w, bar_h);
        let _ = cr.fill();
    }
}
