use std::collections::{HashSet, VecDeque};
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;

use amane::{
    Align, Center, Column, Horizontal, Justify, Layer, LayerWindow, Margin, Notification,
    Notifications, OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Urgency,
    Vertical, Widget, Zone,
};

use crate::config::{ConfigService, ShellManifest};
use crate::services::presentation::PresentationService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

static EXPIRED_POPUPS: LazyLock<Mutex<HashSet<u32>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Dismisses only the on-screen popup toast for this notification,
/// leaving it intact in the Notification Center.
pub fn dismiss_popup(id: u32) {
    if let Ok(mut set) = EXPIRED_POPUPS.lock() {
        set.insert(id);
    }
    amane::request_frame();
}

/// Checks if an on-screen popup toast has been dismissed or expired.
pub fn is_popup_expired(id: u32) -> bool {
    EXPIRED_POPUPS
        .lock()
        .map(|s| s.contains(&id))
        .unwrap_or(false)
}

/// Maximum capacity of suppressed notifications preserved in memory during presentation.
pub const PRESENTATION_BUFFER_MAX_CAPACITY: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferedNotification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
    pub received: SystemTime,
}

impl From<&Notification> for BufferedNotification {
    fn from(n: &Notification) -> Self {
        Self {
            id: n.id(),
            app_name: n.app_name().to_string(),
            summary: n.summary().to_string(),
            body: n.body().to_string(),
            urgency: n.urgency(),
            received: n.received(),
        }
    }
}

static PRESENTATION_BUFFER: LazyLock<Mutex<VecDeque<BufferedNotification>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(PRESENTATION_BUFFER_MAX_CAPACITY)));

/// Append a notification into the FIFO presentation buffer (caps at 50 with oldest evicted).
pub fn buffer_notification(item: BufferedNotification) {
    let mut buffer = match PRESENTATION_BUFFER.lock() {
        Ok(b) => b,
        Err(p) => p.into_inner(),
    };
    if buffer.len() >= PRESENTATION_BUFFER_MAX_CAPACITY {
        buffer.pop_front();
    }
    buffer.push_back(item);
}

/// Drain all buffered notifications sequentially upon presentation exit.
pub fn drain_presentation_buffer() -> Vec<BufferedNotification> {
    let mut buffer = match PRESENTATION_BUFFER.lock() {
        Ok(b) => b,
        Err(p) => p.into_inner(),
    };
    buffer.drain(..).collect()
}

/// Current number of suppressed notifications held in memory.
pub fn presentation_buffer_len() -> usize {
    match PRESENTATION_BUFFER.lock() {
        Ok(b) => b.len(),
        Err(p) => p.into_inner().len(),
    }
}

pub fn notification_popups() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let notifs_cfg = &manifest.notifications;

    if !notifs_cfg.enabled {
        return LayerWindow::new()
            .width(360.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .child(Rectangle::new().width(Parent).height(Parent));
    }

    let presentation_active = PresentationService::global().is_active();

    // In presentation mode, capture non-critical notifications into FIFO buffer before reading for render
    // (dropping the read guard before calling Notifications::dismiss avoids a RwLock self-deadlock).
    if presentation_active {
        let to_dismiss: Vec<u32> = {
            let notifications = Notifications::read();
            let mut buffer = match PRESENTATION_BUFFER.lock() {
                Ok(b) => b,
                Err(p) => p.into_inner(),
            };
            let mut dismiss_ids = Vec::new();
            for n in notifications.list() {
                if n.urgency() != Urgency::Critical {
                    if !buffer.iter().any(|b| b.id == n.id()) {
                        if buffer.len() >= PRESENTATION_BUFFER_MAX_CAPACITY {
                            buffer.pop_front();
                        }
                        buffer.push_back(BufferedNotification::from(n));
                    }
                    dismiss_ids.push(n.id());
                }
            }
            dismiss_ids
        };
        for id in to_dismiss {
            Notifications::dismiss(id);
        }
    }

    let notifications = Notifications::read();
    let notifs_list = notifications.list();
    let cc_service = crate::ui::control_center::ControlCenterService::read();
    let cc_open = cc_service.is_open();
    let dnd_active = cc_service.dnd;

    let expired_popups = match EXPIRED_POPUPS.lock() {
        Ok(e) => e,
        Err(p) => p.into_inner(),
    };
    let now = SystemTime::now();
    let timeout = std::time::Duration::from_millis(notifs_cfg.timeout_ms);

    // Suppress non-critical notification popups in presentation mode, DND, or once toast has expired
    let visible_notifs: Vec<_> = if presentation_active || dnd_active {
        notifs_list
            .iter()
            .filter(|n| n.urgency() == Urgency::Critical && !expired_popups.contains(&n.id()))
            .collect()
    } else {
        notifs_list
            .iter()
            .filter(|n| {
                if expired_popups.contains(&n.id()) {
                    return false;
                }
                if n.urgency() == Urgency::Critical {
                    return true;
                }
                match now.duration_since(n.received()) {
                    Ok(elapsed) => elapsed < timeout,
                    Err(_) => false,
                }
            })
            .collect()
    };

    let target_out = if presentation_active {
        OutputTarget::Named("eDP-1")
    } else {
        OutputTarget::Active
    };

    if visible_notifs.is_empty() || cc_open {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(target_out)
            .visible(false)
            .namespace("dotfield-shell-notifications");
    }

    let max_visible = notifs_cfg.max_visible;
    let mut cards: Vec<Box<dyn Widget>> = Vec::new();
    let font = manifest.theme.font_family.clone();

    for notif in visible_notifs.iter().take(max_visible) {
        cards.push(render_notification_card(notif, &manifest, &font));
    }

    let total_cards_h: f32 = visible_notifs
        .iter()
        .take(max_visible)
        .map(|n| notification_card_height(n))
        .sum();
    let gaps_h = (cards.len().saturating_sub(1) as f32) * spacing::GAP_SM;
    let window_h = (total_cards_h + gaps_h).clamp(96.0, 720.0);

    LayerWindow::new()
        .width(360.0)
        .height(window_h)
        .target_output(target_out)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 14,
            right: 16,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .visible(true)
        .namespace("dotfield-shell-notifications")
        .child(Column::new(cards).width(Parent).gap(spacing::GAP_SM))
}

pub fn notification_card_height(notif: &Notification) -> f32 {
    let mut h = 104.0;
    if !notif.body().is_empty() {
        h += 22.0;
    }
    if !notif.actions().is_empty() {
        h += 30.0;
    }
    h
}

fn render_notification_card(
    notif: &Notification,
    manifest: &ShellManifest,
    font: &str,
) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);
    let id = notif.id();

    let (border_color, header_color) = match notif.urgency() {
        Urgency::Critical => (tokens.border_danger, tokens.danger),
        Urgency::Low => (tokens.border_subtle, tokens.text_muted),
        Urgency::Normal => (tokens.border_subtle, tokens.accent),
    };

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // Header row: App Name + Dismiss button
    let app_text = if notif.app_name().is_empty() {
        "Notification".to_string()
    } else {
        notif.app_name().to_string()
    };

    let header_row = Row::new(vec![
        Box::new(
            Text::new(app_text)
                .size(typography::CAPTION)
                .color(header_color)
                .font(font),
        ),
        Box::new(
            Rectangle::new()
                .width(20.0)
                .height(20.0)
                .radius(radius::PILL)
                .fill(tokens.surface_track)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| {
                    dismiss_popup(id);
                })
                .child(
                    Text::new("󰅖")
                        .size(10.0)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_mono)
                        .tight(),
                ),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(header_row));

    // Summary (bold title)
    elements.push(Box::new(
        Text::new(notif.summary())
            .size(typography::BODY_LG)
            .color(tokens.text_primary)
            .font(font),
    ));

    // Body
    if !notif.body().is_empty() {
        elements.push(Box::new(
            Text::new(notif.body())
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(font),
        ));
    }

    // App Action buttons if any
    if !notif.actions().is_empty() {
        let mut buttons: Vec<Box<dyn Widget>> = Vec::new();
        for action in notif.actions() {
            let key = action.key().to_string();
            let label = action.label().to_string();
            let btn = Rectangle::new()
                .width(80.0)
                .height(24.0)
                .radius(radius::SM)
                .fill(tokens.surface)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| {
                    Notifications::invoke(id, &key);
                    amane::request_frame();
                })
                .child(
                    Text::new(label)
                        .size(typography::BADGE)
                        .color(tokens.text_primary)
                        .font(font),
                );
            buttons.push(Box::new(btn));
        }
        elements.push(Box::new(Row::new(buttons).gap(spacing::GAP_SM)));
    }

    // System Action Buttons: Mark Read, Dismiss, Clean
    let mark_read_btn = Rectangle::new()
        .width(104.0)
        .height(24.0)
        .radius(radius::SM)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| {
            let n_list = Notifications::read();
            let read_key = n_list
                .list()
                .iter()
                .find(|n| n.id() == id)
                .and_then(|target| {
                    target.actions().iter().find(|a| {
                        let k = a.key().to_lowercase();
                        let l = a.label().to_lowercase();
                        k.contains("read") || l.contains("read")
                    })
                })
                .map(|a| a.key().to_string());

            if let Some(key) = read_key {
                Notifications::invoke(id, &key);
            }
            Notifications::dismiss(id);
            amane::request_frame();
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰄬")
                        .size(10.0)
                        .color(tokens.accent)
                        .font(&tokens.font_mono)
                        .tight(),
                ),
                Box::new(
                    Text::new("Mark Read")
                        .size(typography::BADGE)
                        .color(tokens.text_primary)
                        .font(font)
                        .tight(),
                ),
            ])
            .gap(4.0)
            .align(Align::Center),
        );

    let dismiss_btn = Rectangle::new()
        .width(88.0)
        .height(24.0)
        .radius(radius::SM)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| {
            Notifications::dismiss(id);
            amane::request_frame();
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰅖")
                        .size(10.0)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_mono)
                        .tight(),
                ),
                Box::new(
                    Text::new("Dismiss")
                        .size(typography::BADGE)
                        .color(tokens.text_secondary)
                        .font(font)
                        .tight(),
                ),
            ])
            .gap(4.0)
            .align(Align::Center),
        );

    let clean_btn = Rectangle::new()
        .width(76.0)
        .height(24.0)
        .radius(radius::SM)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| {
            Notifications::clear();
            amane::request_frame();
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰃢")
                        .size(10.0)
                        .color(tokens.danger)
                        .font(&tokens.font_mono)
                        .tight(),
                ),
                Box::new(
                    Text::new("Clean")
                        .size(typography::BADGE)
                        .color(tokens.text_secondary)
                        .font(font)
                        .tight(),
                ),
            ])
            .gap(4.0)
            .align(Align::Center),
        );

    let sys_actions = Row::new(vec![
        Box::new(mark_read_btn),
        Box::new(dismiss_btn),
        Box::new(clean_btn),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    elements.push(Box::new(sys_actions));

    let card_h = notification_card_height(notif);
    let card = Rectangle::new()
        .width(Parent)
        .height(card_h)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(1.0, border_color)
        .padding(Padding {
            top: spacing::PAD_SM,
            bottom: spacing::PAD_SM,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .cursor(Pointer)
        .on_click(move |_| {
            Notifications::click(id);
            amane::request_frame();
        })
        .child(Column::new(elements).width(Parent).gap(spacing::GAP_XS));

    Box::new(card)
}

/// Spawns a background thread that automatically expires on-screen popup toasts
/// once their age exceeds the configured `timeout_ms`, while leaving the notification
/// intact in the Notification Center.
pub fn start_expiry_worker() {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(250));

            let manifest = ConfigService::read().manifest.clone();
            if !manifest.notifications.enabled || manifest.notifications.timeout_ms == 0 {
                continue;
            }

            // In presentation mode, pause notification expiry so buffered items are retained
            if PresentationService::global().is_active() {
                continue;
            }

            let timeout = std::time::Duration::from_millis(manifest.notifications.timeout_ms);
            let now = std::time::SystemTime::now();

            let newly_expired: Vec<u32> = {
                let notifs = Notifications::read();
                let expired = match EXPIRED_POPUPS.lock() {
                    Ok(e) => e,
                    Err(p) => p.into_inner(),
                };
                notifs
                    .list()
                    .iter()
                    .filter(|n| {
                        // Critical notifications remain visible as popups until actioned
                        if n.urgency() == Urgency::Critical {
                            return false;
                        }
                        if expired.contains(&n.id()) {
                            return false;
                        }
                        match now.duration_since(n.received()) {
                            Ok(elapsed) => elapsed >= timeout,
                            Err(_) => false,
                        }
                    })
                    .map(|n| n.id())
                    .collect()
            };

            if !newly_expired.is_empty() {
                if let Ok(mut expired) = EXPIRED_POPUPS.lock() {
                    for id in newly_expired {
                        expired.insert(id);
                    }
                }
                amane::request_frame();
            }

            // Prune expired IDs that have been dismissed from Notifications
            let active_ids: HashSet<u32> = Notifications::read()
                .list()
                .iter()
                .map(|n| n.id())
                .collect();
            if let Ok(mut expired) = EXPIRED_POPUPS.lock() {
                expired.retain(|id| active_ids.contains(id));
            }
        }
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_notifications_config_defaults() {
        let manifest = ShellManifest::default();
        ConfigService::write().manifest = manifest.clone();
        assert!(manifest.notifications.enabled);
        assert_eq!(manifest.notifications.timeout_ms, 5000);
        assert_eq!(manifest.notifications.max_visible, 5);
    }

    pub(crate) static NOTIF_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_presentation_fifo_buffer_capping_and_drain() {
        let _guard = NOTIF_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // Ensure clean initial state
        let _ = drain_presentation_buffer();
        assert_eq!(presentation_buffer_len(), 0);

        // Fill buffer past max capacity (50)
        for i in 1..=55 {
            buffer_notification(BufferedNotification {
                id: i,
                app_name: format!("app-{i}"),
                summary: format!("Summary {i}"),
                body: "Body".to_string(),
                urgency: Urgency::Normal,
                received: SystemTime::now(),
            });
        }

        // Must be capped strictly at 50
        assert_eq!(presentation_buffer_len(), PRESENTATION_BUFFER_MAX_CAPACITY);

        // Drain should return 50 items, with oldest (1..=5) evicted
        let drained = drain_presentation_buffer();
        assert_eq!(drained.len(), PRESENTATION_BUFFER_MAX_CAPACITY);
        assert_eq!(drained[0].id, 6);
        assert_eq!(drained.last().unwrap().id, 55);
        assert_eq!(presentation_buffer_len(), 0);
    }

    #[test]
    fn test_popup_dismissal_preserves_notification_center() {
        let _guard = NOTIF_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let test_id = 9999;
        assert!(!is_popup_expired(test_id));
        dismiss_popup(test_id);
        assert!(is_popup_expired(test_id));
    }

    #[test]
    fn test_notification_popups_presentation_mode_no_deadlock() {
        let _guard = NOTIF_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let _ = drain_presentation_buffer();
        Notifications::notify(
            "TestApp",
            "Deadlock check",
            "Must buffer and dismiss without self-deadlocking on RwLock",
            Urgency::Normal,
        );
        let pres = PresentationService::global();
        pres.set_active(true);
        let _win = notification_popups();
        pres.set_active(false);
        let _ = drain_presentation_buffer();
    }
}
