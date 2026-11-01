use parking_lot::RwLock;
use std::sync::LazyLock;

use amane::{
    Center, Color, Column, Horizontal, Layer, LayerWindow, Margin, Notification, Notifications,
    Padding, Parent, Rectangle, Row, Service, Text, Urgency, Vertical, Widget, Zone,
};

use crate::config::ShellManifest;

static CONFIG: LazyLock<RwLock<ShellManifest>> =
    LazyLock::new(|| RwLock::new(ShellManifest::default()));

pub fn set_config(manifest: ShellManifest) {
    *CONFIG.write() = manifest;
}

pub fn notification_popups() -> LayerWindow {
    let manifest = CONFIG.read().clone();
    let notifs_cfg = &manifest.notifications;

    if !notifs_cfg.enabled {
        return LayerWindow::new()
            .width(360.0)
            .height(1.0)
            .visible(false)
            .child(Rectangle::new().width(Parent).height(Parent));
    }

    let notifications = Notifications::read();
    let max_visible = notifs_cfg.max_visible;

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();
    let font = manifest.theme.font_family.clone();

    for notif in notifications.list().iter().take(max_visible) {
        cards.push(render_notification_card(notif, &manifest, &font));
    }

    let is_visible = !cards.is_empty();

    LayerWindow::new()
        .width(360.0)
        .height(600.0)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 40,
            right: 16,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Respect)
        .visible(is_visible)
        .namespace("dotfield-shell-notifications")
        .child(Column::new(cards).width(Parent).gap(10.0))
}

fn render_notification_card(
    notif: &Notification,
    manifest: &ShellManifest,
    font: &str,
) -> Box<dyn Widget> {
    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let crimson_color = Color::from(manifest.palette.accent.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let muted_color = Color::from(manifest.palette.muted.as_str());

    let border_color = match notif.urgency() {
        Urgency::Critical => crimson_color,
        Urgency::Low => muted_color,
        Urgency::Normal => amber_color,
    };

    let id = notif.id();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // Header: App name
    let app_text = if notif.app_name().is_empty() {
        "Notification"
    } else {
        notif.app_name()
    };
    elements.push(Box::new(
        Text::new(app_text)
            .size(10.0)
            .color(border_color)
            .font(font),
    ));

    // Summary (bold title)
    elements.push(Box::new(
        Text::new(notif.summary())
            .size(13.0)
            .color(fg_color)
            .font(font),
    ));

    // Body
    if !notif.body().is_empty() {
        elements.push(Box::new(
            Text::new(notif.body())
                .size(11.0)
                .color(Color::from("#A1A1AA"))
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
                .radius(4.0)
                .fill(Color::from(manifest.palette.overlay.as_str()))
                .align_child(Center, Center)
                .on_click(move |_| Notifications::invoke(id, &key))
                .child(Text::new(label).size(10.0).color(fg_color).font(font));
            buttons.push(Box::new(btn));
        }
        elements.push(Box::new(Row::new(buttons).gap(6.0)));
    }

    let card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(8.0)
        .fill(bg_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 10.0,
            bottom: 10.0,
            left: 14.0,
            right: 14.0,
        })
        .on_click(move |_| Notifications::click(id))
        .child(Column::new(elements).width(Parent).gap(4.0));

    Box::new(card)
}
