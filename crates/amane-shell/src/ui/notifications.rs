use amane::{
    Align, Center, Color, Column, Horizontal, Justify, Layer, LayerWindow, Margin, Notification,
    Notifications, OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Urgency,
    Vertical, Widget, Zone,
};

use crate::config::{ConfigService, ShellManifest};
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

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

    let notifications = Notifications::read();
    let notifs_list = notifications.list();
    let cc_open = crate::ui::control_center::ControlCenterService::read().open;

    if notifs_list.is_empty() || cc_open {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .namespace("dotfield-shell-notifications");
    }

    let max_visible = notifs_cfg.max_visible;
    let mut cards: Vec<Box<dyn Widget>> = Vec::new();
    let font = manifest.theme.font_family.clone();

    for notif in notifs_list.iter().take(max_visible) {
        cards.push(render_notification_card(notif, &manifest, &font));
    }

    let window_h = (cards.len() as f32 * 104.0).clamp(96.0, 520.0);

    LayerWindow::new()
        .width(360.0)
        .height(window_h)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 38,
            right: 16,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Respect)
        .visible(true)
        .namespace("dotfield-shell-notifications")
        .child(Column::new(cards).width(Parent).gap(spacing::GAP_SM))
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
                .width(18.0)
                .height(18.0)
                .radius(radius::XS)
                .fill(Color::TRANSPARENT)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| Notifications::dismiss(id))
                .child(
                    Text::new("󰅖")
                        .size(10.0)
                        .color(tokens.text_muted)
                        .font(font),
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

    // Action buttons if any
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
                .on_click(move |_| Notifications::invoke(id, &key))
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

    let card = Rectangle::new()
        .width(Parent)
        .height(96.0)
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
        .on_click(move |_| Notifications::click(id))
        .child(Column::new(elements).width(Parent).gap(spacing::GAP_XS));

    Box::new(card)
}

/// Spawns a background thread that automatically dismisses non-critical
/// notifications once their age exceeds the configured `timeout_ms`.
pub fn start_expiry_worker() {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(250));

            let manifest = ConfigService::read().manifest.clone();
            if !manifest.notifications.enabled || manifest.notifications.timeout_ms == 0 {
                continue;
            }

            let timeout = std::time::Duration::from_millis(manifest.notifications.timeout_ms);
            let now = std::time::SystemTime::now();

            let to_dismiss: Vec<u32> = {
                let notifs = Notifications::read();
                notifs
                    .list()
                    .iter()
                    .filter(|n| {
                        // Critical notifications remain until clicked
                        if n.urgency() == Urgency::Critical {
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

            for id in to_dismiss {
                Notifications::dismiss(id);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifications_config_defaults() {
        let manifest = ShellManifest::default();
        ConfigService::write().manifest = manifest.clone();
        assert!(manifest.notifications.enabled);
        assert_eq!(manifest.notifications.timeout_ms, 5000);
        assert_eq!(manifest.notifications.max_visible, 5);
    }
}
