use std::time::{Duration, Instant};

use amane::{
    Align, Center, Horizontal, Layer, LayerWindow, Margin, OutputTarget, Padding, Rectangle, Row,
    Service, Text, Vertical, Zone,
};

use crate::config::ConfigService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsdType {
    Volume,
    Brightness,
}

pub struct OsdService {
    pub osd_type: OsdType,
    pub percent: u8,
    pub muted: bool,
    pub visible: bool,
    pub hide_at: Option<Instant>,
}

impl Service for OsdService {
    fn new() -> Self {
        Self {
            osd_type: OsdType::Volume,
            percent: 50,
            muted: false,
            visible: false,
            hide_at: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_millis(50)
    }

    fn update(&mut self) -> bool {
        if self.visible
            && let Some(hide_at) = self.hide_at
            && Instant::now() >= hide_at
        {
            self.visible = false;
            self.hide_at = None;
            return true;
        }
        false
    }
}

pub fn trigger_osd(osd_type: OsdType, percent: u8, muted: bool, timeout_ms: u64) {
    let mut osd = OsdService::write();
    osd.osd_type = osd_type;
    osd.percent = percent;
    osd.muted = muted;
    osd.visible = true;
    osd.hide_at = Some(Instant::now() + Duration::from_millis(timeout_ms));
}

pub fn osd_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let osd = OsdService::read();

    let is_visible = osd.visible && manifest.osd.enabled;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .namespace("dotfield-shell-osd");
    }

    let tokens = crate::ui::tokens::ThemeTokens::from_manifest(&manifest);
    let fill_color = if osd.muted {
        tokens.danger
    } else {
        tokens.accent
    };

    let (icon, label) = match osd.osd_type {
        OsdType::Volume => {
            if osd.muted {
                ("󰖁", "Muted".to_string())
            } else {
                ("󰕾", format!("{}%", osd.percent))
            }
        }
        OsdType::Brightness => ("󰃠", format!("{}%", osd.percent)),
    };

    let gauge = crate::ui::components::progress_bar(
        osd.percent as f32 / 100.0,
        150.0,
        8.0,
        fill_color,
        &tokens,
    );

    let osd_box = Rectangle::new()
        .width(260.0)
        .height(56.0)
        .radius(crate::ui::tokens::radius::LG)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 10.0,
            bottom: 10.0,
            left: 16.0,
            right: 16.0,
        })
        .align_child(Center, Center)
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(icon)
                        .size(crate::ui::tokens::typography::DISPLAY)
                        .color(fill_color)
                        .font(&tokens.font_family),
                ),
                Box::new(gauge),
                Box::new(
                    Text::new(label)
                        .size(crate::ui::tokens::typography::BODY)
                        .color(tokens.text_primary)
                        .font(&tokens.font_family),
                ),
            ])
            .gap(12.0)
            .align(Align::Center),
        );

    LayerWindow::new()
        .width(260.0)
        .height(56.0)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 60,
            bottom: 0,
            left: 0,
            right: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .click_through()
        .visible(is_visible)
        .namespace("dotfield-shell-osd")
        .child(osd_box)
}
