use std::time::{Duration, Instant};

use amane::{
    Align, Center, Color, Horizontal, Layer, LayerWindow, Margin, OutputTarget, Padding, Rectangle,
    Row, Service, Text, Vertical, Zone,
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

    let bg_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let accent_color = Color::from(manifest.palette.accent.as_str()); // Crimson #E63946
    let track_color = Color::from(manifest.palette.muted.as_str()); // Iron #34343D
    let fill_color = if osd.muted {
        accent_color
    } else {
        Color::from(manifest.palette.highlight.as_str()) // Amber #FFB000
    };
    let font = &manifest.theme.font_family;

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

    // Calculate gauge widths
    let track_width = 180.0;
    let fill_width = (track_width * (osd.percent as f32 / 100.0)).clamp(0.0, track_width);

    let gauge = Rectangle::new()
        .width(track_width)
        .height(8.0)
        .radius(4.0)
        .fill(track_color)
        .child(
            Rectangle::new()
                .width(fill_width)
                .height(8.0)
                .radius(4.0)
                .fill(fill_color),
        );

    let osd_box = Rectangle::new()
        .width(260.0)
        .height(60.0)
        .radius(10.0)
        .fill(bg_color)
        .border(1.0, Color::from("#34343D"))
        .padding(Padding {
            top: 10.0,
            bottom: 10.0,
            left: 16.0,
            right: 16.0,
        })
        .align_child(Center, Center)
        .child(
            Row::new(vec![
                Box::new(Text::new(icon).size(20.0).color(fill_color).font(font)),
                Box::new(gauge),
                Box::new(Text::new(label).size(12.0).color(fg_color).font(font)),
            ])
            .gap(12.0)
            .align(Align::Center),
        );

    LayerWindow::new()
        .width(260.0)
        .height(60.0)
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
