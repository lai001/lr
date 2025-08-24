pub use egui;
use egui::{Pos2, Rect, Theme, Vec2, ViewportId, ViewportInfo};
pub use winit;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::ElementState,
};

pub fn screen_size_in_pixels(
    outer_size: PhysicalSize<u32>,
    inner_size: PhysicalSize<u32>,
) -> egui::Vec2 {
    let size = if cfg!(target_os = "ios") {
        outer_size
    } else {
        inner_size
    };
    egui::vec2(size.width as f32, size.height as f32)
}

pub fn pixels_per_point(egui_ctx: &egui::Context, scale_factor: f32) -> f32 {
    let native_pixels_per_point = scale_factor;
    let egui_zoom_factor = egui_ctx.zoom_factor();
    egui_zoom_factor * native_pixels_per_point
}

#[must_use]
#[derive(Clone, Copy, Debug, Default)]
pub struct EventResponse {
    pub consumed: bool,
    pub repaint: bool,
}

pub struct State {
    egui_ctx: egui::Context,
    viewport_id: ViewportId,
    start_time: std::time::Instant,
    egui_input: egui::RawInput,
    pointer_pos_in_points: Option<egui::Pos2>,
    any_pointer_button_down: bool,
    simulate_touch_screen: bool,
    pointer_touch_id: Option<u64>,
    has_sent_ime_enabled: bool,
    allow_ime: bool,
    ime_rect_px: Option<egui::Rect>,
}

impl State {
    pub fn new(
        egui_ctx: egui::Context,
        viewport_id: ViewportId,
        native_pixels_per_point: Option<f32>,
        theme: Option<winit::window::Theme>,
        max_texture_side: Option<usize>,
    ) -> Self {
        let egui_input = egui::RawInput {
            focused: false,
            ..Default::default()
        };

        let mut slf = Self {
            egui_ctx,
            viewport_id,
            start_time: std::time::Instant::now(),
            egui_input,
            pointer_pos_in_points: None,
            any_pointer_button_down: false,
            simulate_touch_screen: false,
            pointer_touch_id: None,
            has_sent_ime_enabled: false,
            allow_ime: false,
            ime_rect_px: None,
        };

        slf.egui_input
            .viewports
            .entry(ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = native_pixels_per_point;
        slf.egui_input.system_theme = theme.map(to_egui_theme);

        if let Some(max_texture_side) = max_texture_side {
            slf.set_max_texture_side(max_texture_side);
        }
        slf
    }

    pub fn set_max_texture_side(&mut self, max_texture_side: usize) {
        self.egui_input.max_texture_side = Some(max_texture_side);
    }

    pub fn allow_ime(&self) -> bool {
        self.allow_ime
    }

    pub fn set_allow_ime(&mut self, allow: bool) {
        self.allow_ime = allow;
    }

    #[inline]
    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }

    #[inline]
    pub fn egui_input(&self) -> &egui::RawInput {
        &self.egui_input
    }

    #[inline]
    pub fn egui_input_mut(&mut self) -> &mut egui::RawInput {
        &mut self.egui_input
    }

    pub fn take_egui_input(
        &mut self,
        outer_size: PhysicalSize<u32>,
        inner_size: PhysicalSize<u32>,
        scale_factor: f32,
    ) -> egui::RawInput {
        self.egui_input.time = Some(self.start_time.elapsed().as_secs_f64());
        let screen_size_in_pixels = screen_size_in_pixels(outer_size, inner_size);
        let screen_size_in_points =
            screen_size_in_pixels / pixels_per_point(&self.egui_ctx, scale_factor);
        self.egui_input.screen_rect = (screen_size_in_points.x > 0.0
            && screen_size_in_points.y > 0.0)
            .then(|| Rect::from_min_size(Pos2::ZERO, screen_size_in_points));
        self.egui_input.viewport_id = self.viewport_id;
        self.egui_input
            .viewports
            .entry(self.viewport_id)
            .or_default()
            .native_pixels_per_point = Some(scale_factor);

        self.egui_input.take()
    }

    pub fn on_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        scale_factor: f32,
    ) -> EventResponse {
        use winit::event::WindowEvent;
        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let native_pixels_per_point = *scale_factor as f32;
                self.egui_input
                    .viewports
                    .entry(self.viewport_id)
                    .or_default()
                    .native_pixels_per_point = Some(native_pixels_per_point);

                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.on_mouse_button_input(*state, *button);
                EventResponse {
                    repaint: true,
                    consumed: self.egui_ctx.wants_pointer_input(),
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.on_mouse_wheel(scale_factor, *delta);
                EventResponse {
                    repaint: true,
                    consumed: self.egui_ctx.wants_pointer_input(),
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.on_cursor_moved(*position, scale_factor);
                EventResponse {
                    repaint: true,
                    consumed: self.egui_ctx.is_using_pointer(),
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.pointer_pos_in_points = None;
                self.egui_input.events.push(egui::Event::PointerGone);
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::Touch(touch) => {
                self.on_touch(touch, scale_factor);
                let consumed = match touch.phase {
                    winit::event::TouchPhase::Started
                    | winit::event::TouchPhase::Ended
                    | winit::event::TouchPhase::Cancelled => self.egui_ctx.wants_pointer_input(),
                    winit::event::TouchPhase::Moved => self.egui_ctx.is_using_pointer(),
                };
                EventResponse {
                    repaint: true,
                    consumed,
                }
            }
            WindowEvent::Ime(ime) => {
                match ime {
                    winit::event::Ime::Enabled => {
                        if cfg!(target_os = "linux") {
                        } else {
                            self.ime_event_enable();
                        }
                    }
                    winit::event::Ime::Preedit(text, Some(_cursor)) => {
                        self.ime_event_enable();
                        self.egui_input
                            .events
                            .push(egui::Event::Ime(egui::ImeEvent::Preedit(text.clone())));
                    }
                    winit::event::Ime::Commit(text) => {
                        self.egui_input
                            .events
                            .push(egui::Event::Ime(egui::ImeEvent::Commit(text.clone())));
                        self.ime_event_disable();
                    }
                    winit::event::Ime::Disabled | winit::event::Ime::Preedit(_, None) => {
                        self.ime_event_disable();
                    }
                };
                EventResponse {
                    repaint: true,
                    consumed: self.egui_ctx.wants_keyboard_input(),
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if *is_synthetic && event.state == ElementState::Pressed {
                    EventResponse {
                        repaint: true,
                        consumed: false,
                    }
                } else {
                    self.on_keyboard_input(event);
                    let consumed = self.egui_ctx.wants_keyboard_input()
                        || event.logical_key
                            == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab);
                    EventResponse {
                        repaint: true,
                        consumed,
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                self.egui_input.focused = *focused;
                self.egui_input
                    .events
                    .push(egui::Event::WindowFocused(*focused));
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::ThemeChanged(winit_theme) => {
                self.egui_input.system_theme = Some(to_egui_theme(*winit_theme));
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::HoveredFile(path) => {
                self.egui_input.hovered_files.push(egui::HoveredFile {
                    path: Some(path.clone()),
                    ..Default::default()
                });
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::HoveredFileCancelled => {
                self.egui_input.hovered_files.clear();
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::DroppedFile(path) => {
                self.egui_input.hovered_files.clear();
                self.egui_input.dropped_files.push(egui::DroppedFile {
                    path: Some(path.clone()),
                    ..Default::default()
                });
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }
            WindowEvent::ModifiersChanged(state) => {
                let state = state.state();
                let alt = state.alt_key();
                let ctrl = state.control_key();
                let shift = state.shift_key();
                let super_ = state.super_key();
                self.egui_input.modifiers.alt = alt;
                self.egui_input.modifiers.ctrl = ctrl;
                self.egui_input.modifiers.shift = shift;
                self.egui_input.modifiers.mac_cmd = cfg!(target_os = "macos") && super_;
                self.egui_input.modifiers.command = if cfg!(target_os = "macos") {
                    super_
                } else {
                    ctrl
                };
                EventResponse {
                    repaint: true,
                    consumed: false,
                }
            }

            WindowEvent::RedrawRequested
            | WindowEvent::CursorEntered { .. }
            | WindowEvent::Destroyed
            | WindowEvent::Occluded(_)
            | WindowEvent::Resized(_)
            | WindowEvent::Moved(_)
            | WindowEvent::TouchpadPressure { .. }
            | WindowEvent::CloseRequested => EventResponse {
                repaint: true,
                consumed: false,
            },

            WindowEvent::ActivationTokenDone { .. }
            | WindowEvent::AxisMotion { .. }
            | WindowEvent::DoubleTapGesture { .. }
            | WindowEvent::RotationGesture { .. }
            | WindowEvent::PanGesture { .. } => EventResponse {
                repaint: false,
                consumed: false,
            },

            WindowEvent::PinchGesture { delta, .. } => {
                let zoom_factor = (*delta as f32).exp();
                self.egui_input.events.push(egui::Event::Zoom(zoom_factor));
                EventResponse {
                    repaint: true,
                    consumed: self.egui_ctx.wants_pointer_input(),
                }
            }
        }
    }

    pub fn ime_event_enable(&mut self) {
        if !self.has_sent_ime_enabled {
            self.egui_input
                .events
                .push(egui::Event::Ime(egui::ImeEvent::Enabled));
            self.has_sent_ime_enabled = true;
        }
    }

    pub fn ime_event_disable(&mut self) {
        self.egui_input
            .events
            .push(egui::Event::Ime(egui::ImeEvent::Disabled));
        self.has_sent_ime_enabled = false;
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.egui_input.events.push(egui::Event::MouseMoved(Vec2 {
            x: delta.0 as f32,
            y: delta.1 as f32,
        }));
    }

    fn on_mouse_button_input(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        if let Some(pos) = self.pointer_pos_in_points {
            if let Some(button) = translate_mouse_button(button) {
                let pressed = state == winit::event::ElementState::Pressed;

                self.egui_input.events.push(egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers: self.egui_input.modifiers,
                });

                if self.simulate_touch_screen {
                    if pressed {
                        self.any_pointer_button_down = true;

                        self.egui_input.events.push(egui::Event::Touch {
                            device_id: egui::TouchDeviceId(0),
                            id: egui::TouchId(0),
                            phase: egui::TouchPhase::Start,
                            pos,
                            force: None,
                        });
                    } else {
                        self.any_pointer_button_down = false;

                        self.egui_input.events.push(egui::Event::PointerGone);

                        self.egui_input.events.push(egui::Event::Touch {
                            device_id: egui::TouchDeviceId(0),
                            id: egui::TouchId(0),
                            phase: egui::TouchPhase::End,
                            pos,
                            force: None,
                        });
                    };
                }
            }
        }
    }

    fn on_cursor_moved(
        &mut self,
        pos_in_pixels: winit::dpi::PhysicalPosition<f64>,
        scale_factor: f32,
    ) {
        let pixels_per_point = pixels_per_point(&self.egui_ctx, scale_factor);

        let pos_in_points = egui::pos2(
            pos_in_pixels.x as f32 / pixels_per_point,
            pos_in_pixels.y as f32 / pixels_per_point,
        );
        self.pointer_pos_in_points = Some(pos_in_points);

        if self.simulate_touch_screen {
            if self.any_pointer_button_down {
                self.egui_input
                    .events
                    .push(egui::Event::PointerMoved(pos_in_points));

                self.egui_input.events.push(egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId(0),
                    phase: egui::TouchPhase::Move,
                    pos: pos_in_points,
                    force: None,
                });
            }
        } else {
            self.egui_input
                .events
                .push(egui::Event::PointerMoved(pos_in_points));
        }
    }

    fn on_touch(&mut self, touch: &winit::event::Touch, scale_factor: f32) {
        let pixels_per_point = pixels_per_point(&self.egui_ctx, scale_factor);

        self.egui_input.events.push(egui::Event::Touch {
            device_id: egui::TouchDeviceId(egui::epaint::util::hash(touch.device_id)),
            id: egui::TouchId::from(touch.id),
            phase: match touch.phase {
                winit::event::TouchPhase::Started => egui::TouchPhase::Start,
                winit::event::TouchPhase::Moved => egui::TouchPhase::Move,
                winit::event::TouchPhase::Ended => egui::TouchPhase::End,
                winit::event::TouchPhase::Cancelled => egui::TouchPhase::Cancel,
            },
            pos: egui::pos2(
                touch.location.x as f32 / pixels_per_point,
                touch.location.y as f32 / pixels_per_point,
            ),
            force: match touch.force {
                Some(winit::event::Force::Normalized(force)) => Some(force as f32),
                Some(winit::event::Force::Calibrated {
                    force,
                    max_possible_force,
                    ..
                }) => Some((force / max_possible_force) as f32),
                None => None,
            },
        });
        if self.pointer_touch_id.is_none() || self.pointer_touch_id.unwrap_or_default() == touch.id
        {
            match touch.phase {
                winit::event::TouchPhase::Started => {
                    self.pointer_touch_id = Some(touch.id);
                    self.on_cursor_moved(touch.location, scale_factor);
                    self.on_mouse_button_input(
                        winit::event::ElementState::Pressed,
                        winit::event::MouseButton::Left,
                    );
                }
                winit::event::TouchPhase::Moved => {
                    self.on_cursor_moved(touch.location, scale_factor);
                }
                winit::event::TouchPhase::Ended => {
                    self.pointer_touch_id = None;
                    self.on_mouse_button_input(
                        winit::event::ElementState::Released,
                        winit::event::MouseButton::Left,
                    );
                    self.pointer_pos_in_points = None;
                    self.egui_input.events.push(egui::Event::PointerGone);
                }
                winit::event::TouchPhase::Cancelled => {
                    self.pointer_touch_id = None;
                    self.pointer_pos_in_points = None;
                    self.egui_input.events.push(egui::Event::PointerGone);
                }
            }
        }
    }

    fn on_mouse_wheel(&mut self, scale_factor: f32, delta: winit::event::MouseScrollDelta) {
        let pixels_per_point = pixels_per_point(&self.egui_ctx, scale_factor);
        {
            let (unit, delta) = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    (egui::MouseWheelUnit::Line, egui::vec2(x, y))
                }
                winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
                    x,
                    y,
                }) => (
                    egui::MouseWheelUnit::Point,
                    egui::vec2(x as f32, y as f32) / pixels_per_point,
                ),
            };
            let modifiers = self.egui_input.modifiers;
            self.egui_input.events.push(egui::Event::MouseWheel {
                unit,
                delta,
                modifiers,
            });
        }
    }

    fn on_keyboard_input(&mut self, event: &winit::event::KeyEvent) {
        let winit::event::KeyEvent {
            physical_key,
            logical_key: winit_logical_key,
            text,
            state,
            location: _,
            repeat: _,
            ..
        } = event;
        let pressed = *state == winit::event::ElementState::Pressed;
        let physical_key = if let winit::keyboard::PhysicalKey::Code(keycode) = *physical_key {
            key_from_key_code(keycode)
        } else {
            None
        };
        let logical_key = key_from_winit_key(winit_logical_key);
        log::trace!(
            "logical {:?} -> {:?},  physical {:?} -> {:?}",
            event.logical_key,
            logical_key,
            event.physical_key,
            physical_key
        );
        if let Some(active_key) = logical_key.or(physical_key) {
            if pressed {
                if is_cut_command(self.egui_input.modifiers, active_key) {
                    self.egui_input.events.push(egui::Event::Cut);
                    return;
                } else if is_copy_command(self.egui_input.modifiers, active_key) {
                    self.egui_input.events.push(egui::Event::Copy);
                    return;
                } else if is_paste_command(self.egui_input.modifiers, active_key) {
                    return;
                }
            }

            self.egui_input.events.push(egui::Event::Key {
                key: active_key,
                physical_key,
                pressed,
                repeat: false,
                modifiers: self.egui_input.modifiers,
            });
        }

        if let Some(text) = text
            .as_ref()
            .map(|t| t.as_str())
            .or_else(|| winit_logical_key.to_text())
        {
            if !text.is_empty() && text.chars().all(is_printable_char) {
                let is_cmd = self.egui_input.modifiers.ctrl
                    || self.egui_input.modifiers.command
                    || self.egui_input.modifiers.mac_cmd;
                if pressed && !is_cmd {
                    self.egui_input
                        .events
                        .push(egui::Event::Text(text.to_owned()));
                }
            }
        }
    }

    pub fn handle_platform_output(
        &mut self,
        platform_output: egui::PlatformOutput,
        scale_factor: f32,
    ) {
        let egui::PlatformOutput {
            commands,
            cursor_icon: _,
            #[allow(deprecated)]
            open_url,
            #[allow(deprecated)]
                copied_text: _,
            events: _,
            mutable_text_under_cursor: _,
            ime,
            num_completed_passes: _,
            request_discard_reasons: _,
        } = platform_output;

        for command in commands {
            match command {
                egui::OutputCommand::OpenUrl(open_url) => {
                    open_url_in_browser(&open_url.url);
                }
                _ => {}
            }
        }

        if let Some(open_url) = open_url {
            open_url_in_browser(&open_url.url);
        }

        let allow_ime = ime.is_some();
        if self.allow_ime != allow_ime {
            self.allow_ime = allow_ime;
        }

        if let Some(ime) = ime {
            let pixels_per_point = pixels_per_point(&self.egui_ctx, scale_factor);
            let ime_rect_px = pixels_per_point * ime.rect;
            if self.ime_rect_px != Some(ime_rect_px)
                || self.egui_ctx.input(|i| !i.events.is_empty())
            {
                self.ime_rect_px = Some(ime_rect_px);
            }
        } else {
            self.ime_rect_px = None;
        }
    }
}

fn to_egui_theme(theme: winit::window::Theme) -> Theme {
    match theme {
        winit::window::Theme::Dark => Theme::Dark,
        winit::window::Theme::Light => Theme::Light,
    }
}

pub fn inner_rect_in_points(
    inner_position: PhysicalPosition<i32>,
    inner_size: PhysicalSize<u32>,
    pixels_per_point: f32,
) -> Option<Rect> {
    let inner_pos_px = inner_position;
    let inner_pos_px = egui::pos2(inner_pos_px.x as f32, inner_pos_px.y as f32);
    let inner_size_px = inner_size;
    let inner_size_px = egui::vec2(inner_size_px.width as f32, inner_size_px.height as f32);
    let inner_rect_px = egui::Rect::from_min_size(inner_pos_px, inner_size_px);
    Some(inner_rect_px / pixels_per_point)
}

pub fn outer_rect_in_points(
    outer_position: PhysicalPosition<i32>,
    outer_size: PhysicalSize<u32>,
    pixels_per_point: f32,
) -> Option<Rect> {
    let outer_pos_px = outer_position;
    let outer_pos_px = egui::pos2(outer_pos_px.x as f32, outer_pos_px.y as f32);
    let outer_size_px = outer_size;
    let outer_size_px = egui::vec2(outer_size_px.width as f32, outer_size_px.height as f32);
    let outer_rect_px = egui::Rect::from_min_size(outer_pos_px, outer_size_px);
    Some(outer_rect_px / pixels_per_point)
}

pub fn update_viewport_info(
    viewport_info: &mut ViewportInfo,
    egui_ctx: &egui::Context,
    is_init: bool,
    scale_factor: f32,
    is_minimized: Option<bool>,
    is_maximized: bool,
    inner_position: PhysicalPosition<i32>,
    inner_size: PhysicalSize<u32>,
    outer_position: PhysicalPosition<i32>,
    outer_size: PhysicalSize<u32>,
    title: Option<String>,
    monitor: Option<winit::monitor::MonitorHandle>,
    fullscreen: Option<winit::window::Fullscreen>,
    has_focus: bool,
) {
    let pixels_per_point = pixels_per_point(egui_ctx, scale_factor);

    let has_a_position = match is_minimized {
        Some(true) => false,
        Some(false) | None => true,
    };

    let inner_rect = if has_a_position {
        inner_rect_in_points(inner_position, inner_size, pixels_per_point)
    } else {
        None
    };

    let outer_rect = if has_a_position {
        outer_rect_in_points(outer_position, outer_size, pixels_per_point)
    } else {
        None
    };

    let monitor_size = {
        if let Some(monitor) = monitor {
            let size = monitor.size().to_logical::<f32>(pixels_per_point.into());
            Some(egui::vec2(size.width, size.height))
        } else {
            None
        }
    };

    viewport_info.title = title;
    viewport_info.native_pixels_per_point = Some(scale_factor);

    viewport_info.monitor_size = monitor_size;
    viewport_info.inner_rect = inner_rect;
    viewport_info.outer_rect = outer_rect;

    if is_init || !cfg!(target_os = "macos") {
        viewport_info.maximized = Some(is_maximized);
        viewport_info.minimized = Some(is_minimized.unwrap_or(false));
    }

    viewport_info.fullscreen = Some(fullscreen.is_some());
    viewport_info.focused = Some(has_focus);
}

fn open_url_in_browser(_url: &str) {
    unimplemented!();
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn is_cut_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
    keycode == egui::Key::Cut || (modifiers.command && keycode == egui::Key::X)
}

fn is_copy_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
    keycode == egui::Key::Copy || (modifiers.command && keycode == egui::Key::C)
}

fn is_paste_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
    keycode == egui::Key::Paste || (modifiers.command && keycode == egui::Key::V)
}

fn translate_mouse_button(button: winit::event::MouseButton) -> Option<egui::PointerButton> {
    match button {
        winit::event::MouseButton::Left => Some(egui::PointerButton::Primary),
        winit::event::MouseButton::Right => Some(egui::PointerButton::Secondary),
        winit::event::MouseButton::Middle => Some(egui::PointerButton::Middle),
        winit::event::MouseButton::Back => Some(egui::PointerButton::Extra1),
        winit::event::MouseButton::Forward => Some(egui::PointerButton::Extra2),
        winit::event::MouseButton::Other(_) => None,
    }
}

fn key_from_winit_key(key: &winit::keyboard::Key) -> Option<egui::Key> {
    match key {
        winit::keyboard::Key::Named(named_key) => key_from_named_key(*named_key),
        winit::keyboard::Key::Character(str) => egui::Key::from_name(str.as_str()),
        winit::keyboard::Key::Unidentified(_) | winit::keyboard::Key::Dead(_) => None,
    }
}

fn key_from_named_key(named_key: winit::keyboard::NamedKey) -> Option<egui::Key> {
    use egui::Key;
    use winit::keyboard::NamedKey;

    Some(match named_key {
        NamedKey::Enter => Key::Enter,
        NamedKey::Tab => Key::Tab,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::End => Key::End,
        NamedKey::Home => Key::Home,
        NamedKey::PageDown => Key::PageDown,
        NamedKey::PageUp => Key::PageUp,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Delete => Key::Delete,
        NamedKey::Insert => Key::Insert,
        NamedKey::Escape => Key::Escape,
        NamedKey::Cut => Key::Cut,
        NamedKey::Copy => Key::Copy,
        NamedKey::Paste => Key::Paste,

        NamedKey::Space => Key::Space,

        NamedKey::F1 => Key::F1,
        NamedKey::F2 => Key::F2,
        NamedKey::F3 => Key::F3,
        NamedKey::F4 => Key::F4,
        NamedKey::F5 => Key::F5,
        NamedKey::F6 => Key::F6,
        NamedKey::F7 => Key::F7,
        NamedKey::F8 => Key::F8,
        NamedKey::F9 => Key::F9,
        NamedKey::F10 => Key::F10,
        NamedKey::F11 => Key::F11,
        NamedKey::F12 => Key::F12,
        NamedKey::F13 => Key::F13,
        NamedKey::F14 => Key::F14,
        NamedKey::F15 => Key::F15,
        NamedKey::F16 => Key::F16,
        NamedKey::F17 => Key::F17,
        NamedKey::F18 => Key::F18,
        NamedKey::F19 => Key::F19,
        NamedKey::F20 => Key::F20,
        NamedKey::F21 => Key::F21,
        NamedKey::F22 => Key::F22,
        NamedKey::F23 => Key::F23,
        NamedKey::F24 => Key::F24,
        NamedKey::F25 => Key::F25,
        NamedKey::F26 => Key::F26,
        NamedKey::F27 => Key::F27,
        NamedKey::F28 => Key::F28,
        NamedKey::F29 => Key::F29,
        NamedKey::F30 => Key::F30,
        NamedKey::F31 => Key::F31,
        NamedKey::F32 => Key::F32,
        NamedKey::F33 => Key::F33,
        NamedKey::F34 => Key::F34,
        NamedKey::F35 => Key::F35,

        NamedKey::BrowserBack => Key::BrowserBack,
        _ => {
            log::trace!("Unknown key: {named_key:?}");
            return None;
        }
    })
}

fn key_from_key_code(key: winit::keyboard::KeyCode) -> Option<egui::Key> {
    use egui::Key;
    use winit::keyboard::KeyCode;

    Some(match key {
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ArrowUp => Key::ArrowUp,

        KeyCode::Escape => Key::Escape,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter | KeyCode::NumpadEnter => Key::Enter,

        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,

        KeyCode::Space => Key::Space,
        KeyCode::Comma => Key::Comma,
        KeyCode::Period => Key::Period,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::Slash | KeyCode::NumpadDivide => Key::Slash,
        KeyCode::BracketLeft => Key::OpenBracket,
        KeyCode::BracketRight => Key::CloseBracket,
        KeyCode::Backquote => Key::Backtick,
        KeyCode::Quote => Key::Quote,

        KeyCode::Cut => Key::Cut,
        KeyCode::Copy => Key::Copy,
        KeyCode::Paste => Key::Paste,
        KeyCode::Minus | KeyCode::NumpadSubtract => Key::Minus,
        KeyCode::NumpadAdd => Key::Plus,
        KeyCode::Equal => Key::Equals,

        KeyCode::Digit0 | KeyCode::Numpad0 => Key::Num0,
        KeyCode::Digit1 | KeyCode::Numpad1 => Key::Num1,
        KeyCode::Digit2 | KeyCode::Numpad2 => Key::Num2,
        KeyCode::Digit3 | KeyCode::Numpad3 => Key::Num3,
        KeyCode::Digit4 | KeyCode::Numpad4 => Key::Num4,
        KeyCode::Digit5 | KeyCode::Numpad5 => Key::Num5,
        KeyCode::Digit6 | KeyCode::Numpad6 => Key::Num6,
        KeyCode::Digit7 | KeyCode::Numpad7 => Key::Num7,
        KeyCode::Digit8 | KeyCode::Numpad8 => Key::Num8,
        KeyCode::Digit9 | KeyCode::Numpad9 => Key::Num9,

        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,

        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,
        KeyCode::F21 => Key::F21,
        KeyCode::F22 => Key::F22,
        KeyCode::F23 => Key::F23,
        KeyCode::F24 => Key::F24,
        KeyCode::F25 => Key::F25,
        KeyCode::F26 => Key::F26,
        KeyCode::F27 => Key::F27,
        KeyCode::F28 => Key::F28,
        KeyCode::F29 => Key::F29,
        KeyCode::F30 => Key::F30,
        KeyCode::F31 => Key::F31,
        KeyCode::F32 => Key::F32,
        KeyCode::F33 => Key::F33,
        KeyCode::F34 => Key::F34,
        KeyCode::F35 => Key::F35,

        _ => {
            return None;
        }
    })
}
