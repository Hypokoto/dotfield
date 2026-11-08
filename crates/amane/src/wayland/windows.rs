use std::collections::HashSet;

use wayland_client::{
    Proxy,
    protocol::{wl_output::WlOutput, wl_surface::WlSurface},
};

use crate::graphics::Gpu;
use crate::input::Pointer;
use crate::{LayerWindow, OutputTarget, Service, changes, frame};

use super::{
    WaylandState,
    layer::{self, Settings},
    surface::{Content, OpenWindow, Role, View},
};

impl WaylandState {
    pub fn open(&mut self, view: View, output: Option<WlOutput>) {
        let surface = self.compositor.create_surface(&self.qh);

        /*
         * later views can change the settings, each redraw compares them with the
         * last ones; a window that starts hidden never gets a configure, so it never
         * draws, and what its first view read is all that can wake it to show
         */
        let (content, reads) = frame::run_view(|| view.run(), 0, 0, self.scale_factor);

        // normal windows open in open_normal
        let Content::Layer(window) = content else {
            unreachable!("only layer views open as layer windows");
        };

        let settings = Settings::from(&window);

        let layer_surface = layer::create(
            &self.layer_shell,
            surface,
            output.as_ref(),
            &self.qh,
            &settings,
            self.scale_factor,
        );

        let role = Role::Layer {
            surface: layer_surface,
            settings,
        };

        self.add(view, output, role);

        if let Some(window) = self.windows.last_mut() {
            window.reads = reads;
        }
    }

    // everything a window needs besides its surface is the same for layers and lock screens
    pub fn add(&mut self, view: View, output: Option<WlOutput>, role: Role) {
        // the gpu draws straight into the surface, so it gets libwayland's own pointers
        let display = self.connection.backend().display_ptr().cast();
        let surface = role.wl_surface().id().as_ptr().cast();

        let gpu = Gpu::new(display, surface);

        let fractional = self.make_fractional(role.wl_surface());

        let window = OpenWindow {
            view,

            output,

            input_region: None,

            width: 0,
            height: 0,

            scale: 1.0,

            scale_factor: self.scale_factor,

            frame_requested: false,
            last_frame: None,
            reads: HashSet::new(),

            pointer: Pointer::default(),
            on_key: None,

            gpu,

            fractional,

            role,

            compositor: self.compositor.wl_compositor().clone(),

            qh: self.qh.clone(),
        };

        self.windows.push(window);
    }

    // wayland events name the surface they are about, this finds its window
    pub fn window(&mut self, surface: &WlSurface) -> Option<&mut OpenWindow> {
        self.windows
            .iter_mut()
            .find(|window| window.role.wl_surface() == surface)
    }

    // a service or handler may have changed what any of the windows shows
    pub fn request_frames(&mut self) {
        for window in &mut self.windows {
            window.request_frame();
        }
    }

    // input only changes the window it arrived in, besides the services it wrote
    pub fn request_frame_on(&mut self, surface: &WlSurface) {
        if let Some(window) = self.window(surface) {
            window.request_frame();
        }
    }

    // only windows that read a service that changed draw again
    pub fn request_changed_frames(&mut self) {
        let Some(changed) = changes::take() else {
            self.request_frames();

            return;
        };

        for window in &mut self.windows {
            if window.reads.is_disjoint(&changed) {
                continue;
            }

            window.request_frame();
        }
    }

    pub fn close(&mut self, surface: &WlSurface) {
        let closed_window = self
            .windows
            .iter()
            .find(|w| w.role.wl_surface() == surface)
            .map(|w| (w.view.clone(), w.output.clone()));

        self.windows
            .retain(|window| window.role.wl_surface() != surface);

        // If a plain layer window was closed by the compositor (e.g. output renegotiation/sleep),
        // and an active output is still available, recreate the window so overlays are never lost
        if let Some((View::Plain { id, view }, _)) = closed_window {
            let active_output = self.output.outputs().next();
            if let Some(output) = active_output {
                self.open(View::Plain { id, view }, Some(output));
            }
        }

        // windows made per monitor or plain views come back when a monitor is plugged in again.
        // A shell daemon with IPC also keeps running.
        if self.windows.is_empty()
            && self.per_monitor.is_empty()
            && self.plain_views.is_empty()
            && !self.has_ipc
        {
            self.running = false;
        }

        // free the memory the closed windows held, instead of waiting for the next trim pass
        #[cfg(target_env = "gnu")]
        crate::allocator::trim_once();
    }

    /// Ensures that all registered plain layer views (launcher, display config, control center, etc.)
    /// have an active window surface. Recreates any missing views on the active output.
    pub fn ensure_plain_windows(&mut self) {
        let plain_views = self.plain_views.clone();
        for (id, plain) in plain_views.into_iter().enumerate() {
            let already_open = self.windows.iter().any(|w| {
                if let View::Plain {
                    id: existing_id, ..
                } = w.view
                {
                    existing_id == id
                } else {
                    false
                }
            });
            if !already_open {
                let active = self.current_focused_output();
                self.open(View::Plain { id, view: plain }, active);
            }
        }
    }

    /// Returns the currently active output for placing overlays:
    /// 1. The output corresponding to the compositor's currently focused workspace (fast cached lookup).
    /// 2. The output tracked from recent pointer/keyboard interaction on an Amane window.
    /// 3. The output actively focused by the Wayland compositor via IPC query.
    /// 4. If neither is available, falls back to the first available output.
    pub fn current_focused_output(&self) -> Option<WlOutput> {
        let focused_ws_output = crate::Workspaces::read()
            .list()
            .iter()
            .find(|ws| ws.focused())
            .and_then(|ws| ws.output().map(String::from));

        if let Some(matched) = focused_ws_output.and_then(|ws_name| {
            self.output
                .outputs()
                .find(|out| self.output.info(out).and_then(|i| i.name).as_deref() == Some(&ws_name))
        }) {
            return Some(matched);
        }

        if let Some(focused) = self
            .focused_output
            .as_ref()
            .filter(|f| self.output.outputs().any(|o| &o == *f))
        {
            return Some(focused.clone());
        }

        if let Some(matched) = crate::compositor::focused_output().and_then(|comp_output| {
            self.output
                .outputs()
                .find(|out| self.output.info(out).and_then(|i| i.name).as_deref() == Some(&comp_output))
        }) {
            return Some(matched);
        }

        self.output.outputs().next()
    }

    // Move any layer overlay requesting OutputTarget::Active that is visible but on the wrong output
    // to the active output, so overlays appear on the external display when it has focus.
    pub fn relocate_plain_overlays(&mut self) {
        self.ensure_plain_windows();

        let has_active_visible = self.windows.iter().any(|w| {
            if let View::Plain { .. } = w.view
                && let Role::Layer { settings, .. } = &w.role
            {
                settings.visible && matches!(settings.target_output, OutputTarget::Active)
            } else {
                false
            }
        });

        if !has_active_visible {
            return;
        }

        let Some(active) = self.current_focused_output() else {
            return;
        };

        type PlainMoveTarget = (usize, fn() -> LayerWindow, WlOutput);

        // collect plain windows that are visible and on a different output than targeted
        let to_move: Vec<PlainMoveTarget> = self
            .windows
            .iter()
            .filter_map(|w| {
                let View::Plain { id, view } = w.view else {
                    return None;
                };
                let Role::Layer { settings, .. } = &w.role else {
                    return None;
                };
                if !settings.visible {
                    return None;
                }

                let target_output = match settings.target_output {
                    OutputTarget::Active => active.clone(),
                    OutputTarget::Named(name) => self.output.outputs().find(|out| {
                        self.output.info(out).and_then(|i| i.name) == Some(name.to_string())
                    })?,
                    OutputTarget::Default => return None,
                };

                // Target output already matches: nothing to do
                if w.output.as_ref() == Some(&target_output) {
                    return None;
                }

                Some((id, view, target_output))
            })
            .collect();

        for (id, view, target) in to_move {
            // Unmap old surface cleanly from previous display before dropping
            if let Some(w) = self.windows.iter_mut().find(|w| {
                if let View::Plain { id: wid, .. } = w.view {
                    wid == id
                } else {
                    false
                }
            }) {
                w.hide();
            }

            // drop the window on the wrong output
            self.windows.retain(|w| {
                if let View::Plain { id: wid, .. } = w.view {
                    wid != id
                } else {
                    true
                }
            });
            // reopen on the target output
            self.open(View::Plain { id, view }, Some(target));
        }
    }
}
