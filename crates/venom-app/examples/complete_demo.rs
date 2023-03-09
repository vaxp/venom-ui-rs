//! VenomUI Complete Widget Showcase
//!
//! Demonstrates ALL widgets in a single interactive application.
//!
//! Run with: cargo run --example complete_demo

use venom_app::{App, AppConfig, RenderContext};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point, BorderRadius, Offset};
use venom_widgets::{Shell, Cursor, Widget};
use std::time::Instant;

// ============================================================================
// APPLICATION STATE
// ============================================================================

struct AppState {
    // Checkboxes
    checkbox1: bool,
    checkbox2: bool,
    checkbox3: bool,
    
    // Switches
    switch1: bool,
    switch2: bool,
    dark_mode: bool,
    
    // Sliders
    volume: f32,
    brightness: f32,
    
    // Progress
    download_progress: f32,
    upload_progress: f32,
    
    // Text fields (simulated)
    username: String,
    password: String,
    
    // Buttons
    button_clicks: [u32; 4],
    
    // Dialog
    show_dialog: bool,
    
    // State
    messages: Vec<String>,
    scroll_offset: f32,
    dragging_slider: Option<usize>,
    last_click: u64,
    last_frame_time: Option<Instant>,
    
    // Animation
    frame: u64,
    
    // Input state
    was_pressed: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            checkbox1: true,
            checkbox2: false,
            checkbox3: true,
            
            switch1: true,
            switch2: false,
            dark_mode: true,
            
            volume: 75.0,
            brightness: 50.0,
            
            download_progress: 0.0,
            upload_progress: 0.6,
            
            username: "user@example.com".to_string(),
            password: "••••••••".to_string(),
            
            button_clicks: [0; 4],
            
            show_dialog: false,
            
            messages: vec![],
            scroll_offset: 0.0,
            dragging_slider: None,
            last_click: 0,
            last_frame_time: None,
            
            frame: 0,
            was_pressed: false,
        }
    }

    fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::dark()
        } else {
            Theme::light()
        }
    }
}

struct Theme {
    bg: Color,
    card: Color,
    text: Color,
    muted: Color,
    accent: Color,
    success: Color,
    warning: Color,
    danger: Color,
}

impl Theme {
    fn dark() -> Self {
        Self {
            bg: Color::hex("#0f172a"),
            card: Color::hex("#1e293b"),
            text: Color::WHITE,
            muted: Color::hex("#64748b"),
            accent: Color::hex("#6366f1"),
            success: Color::hex("#22c55e"),
            warning: Color::hex("#f97316"),
            danger: Color::hex("#ef4444"),
        }
    }
    
    fn light() -> Self {
        Self {
            bg: Color::hex("#f1f5f9"),
            card: Color::WHITE,
            text: Color::hex("#0f172a"),
            muted: Color::hex("#64748b"),
            accent: Color::hex("#4f46e5"),
            success: Color::hex("#16a34a"),
            warning: Color::hex("#ea580c"),
            danger: Color::hex("#dc2626"),
        }
    }
}

// Global state
static mut STATE: Option<AppState> = None;

fn get_state() -> &'static mut AppState {
    unsafe {
        if STATE.is_none() {
            STATE = Some(AppState::new());
        }
        STATE.as_mut().unwrap()
    }
}

fn main() {
    println!("🎨 VenomUI Complete Widget Showcase");
    println!("====================================");
    println!("Interactive demo of all widgets\n");
    
    let mut config = AppConfig::new("VenomUI Complete Showcase")
            .size(1000, 700)
            .background(Color::hex("#0f172a"));
    
    // Unlock FPS
    config.vsync = false;
            
    App::run(
        config,
        render,
    );
}

fn render(canvas: &mut SoftwareBackend, ctx: &RenderContext) {
    let state = get_state();
    state.frame = ctx.frame;
    
    // Animate progress
    state.download_progress = ((ctx.frame as f32 * 0.01).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    
    let theme = state.theme();
    let cursor = ctx.mouse_position.map(|p| Cursor::at(p.x, p.y)).unwrap_or(Cursor::unavailable());

    // Handle input
    handle_input(state, cursor, ctx.mouse_pressed, ctx.frame);

    // Draw background
    canvas.draw_rect(Rect::new(0.0, 0.0, ctx.width as f32, ctx.height as f32), &Paint::fill(theme.bg));

    // Header
    draw_header(canvas, &theme, ctx.width as f32);

    // Main content - 3 columns
    let col_width = 300.0;
    let padding = 20.0;
    let start_y = 80.0;

    // Column 1: Form Controls
    draw_form_controls(canvas, state, &theme, cursor, padding, start_y, col_width);

    // Column 2: Sliders & Progress
    draw_sliders_progress(canvas, state, &theme, cursor, padding + col_width + 20.0, start_y, col_width);

    // Column 3: Buttons & Info
    draw_buttons_info(canvas, state, &theme, cursor, padding + (col_width + 20.0) * 2.0, start_y, col_width);

    // Dialog overlay
    if state.show_dialog {
        draw_dialog(canvas, state, &theme, cursor, ctx.width as f32, ctx.height as f32);
    }

    // Calculate real FPS
    let now = Instant::now();
    let real_dt = if let Some(last) = state.last_frame_time {
        now.duration_since(last).as_secs_f32()
    } else {
        0.016 // First frame assumption
    };
    state.last_frame_time = Some(now);

    // FPS Counter - print every 60 frames
    if ctx.frame % 60 == 0 {
        let fps = if real_dt > 0.0 { 1.0 / real_dt } else { 0.0 };
        println!("📊 Frame: {} | FPS: {:.0} | {:.2}ms/frame", ctx.frame, fps, real_dt * 1000.0);
    }
    
    // FPS visual indicator (green box in corner)
    canvas.draw_rounded_rect(
        Rect::new(ctx.width as f32 - 80.0, 10.0, 70.0, 20.0),
        BorderRadius::all(4.0),
        &Paint::fill(Color::hex("#22c55e")),
    );

    // Mouse cursor
    if let Some(pos) = ctx.mouse_position {
        canvas.draw_circle(pos, 6.0, &Paint::fill(Color::rgba(99, 102, 241, 200)));
        canvas.draw_circle(pos, 6.0, &Paint::stroke(Color::WHITE, 1.5));
    }

    // Request animation frame
    // We only need to redraw the sliders column where animation happens
    // Column 2 coordinates:
    let col_width = 300.0;
    let padding = 20.0;
    let start_y = 80.0;
    let x = padding + col_width + 20.0;
    let y = start_y;
    let width = col_width;
    let height = 320.0;
    
    ctx.request_redraw(Some(Rect::new(x, y, width, height)));
}

fn handle_input(state: &mut AppState, cursor: Cursor, pressed: bool, frame: u64) {
    // Track mouse state for rising edge detection
    let just_pressed = pressed && !state.was_pressed;
    state.was_pressed = pressed;

    if !just_pressed && !state.dragging_slider.is_some() {
        return;
    }

    if !pressed {
        state.dragging_slider = None;
        return;
    }

    let Some((mx, my)) = cursor.position else { return };

    // Layout constants - must match draw functions
    let col_width = 300.0;
    let padding = 20.0;
    let start_y = 80.0;
    
    // Column 1: Form Controls
    let col1_x = padding;
    let col1_px = col1_x + 15.0;
    
    // Column 2: Sliders
    let col2_x = padding + col_width + 20.0;
    let col2_px = col2_x + 15.0;
    let slider_width = col_width - 30.0;
    
    // Column 3: Buttons
    let col3_x = padding + (col_width + 20.0) * 2.0;
    let col3_px = col3_x + 15.0;

    // Slider positions (matching draw_sliders_progress)
    let slider1_y = start_y + 15.0 + 30.0 + 20.0; // card y + title + label + spacing
    let slider2_y = slider1_y + 45.0 + 20.0;       // slider1_y + 45 + label spacing

    // Handle slider dragging - always process if dragging OR if just clicked on slider
    if state.dragging_slider == Some(0) || 
       (just_pressed && mx >= col2_px && mx <= col2_px + slider_width && my >= slider1_y && my <= slider1_y + 20.0) {
        if pressed && mx >= col2_px && mx <= col2_px + slider_width {
            state.dragging_slider = Some(0);
            state.volume = ((mx - col2_px) / slider_width * 100.0).clamp(0.0, 100.0);
            return;
        }
    }

    if state.dragging_slider == Some(1) ||
       (just_pressed && mx >= col2_px && mx <= col2_px + slider_width && my >= slider2_y && my <= slider2_y + 20.0) {
        if pressed && mx >= col2_px && mx <= col2_px + slider_width {
            state.dragging_slider = Some(1);
            state.brightness = ((mx - col2_px) / slider_width * 100.0).clamp(0.0, 100.0);
            return;
        }
    }

    // For everything else, strictly require just_pressed
    if !just_pressed {
        return;
    }

    // Checkbox positions (matching draw_form_controls)
    let cb_base_y = start_y + 15.0 + 30.0 + 20.0; // card y + title + label + spacing
    if cursor.is_over(col1_px, cb_base_y, 24.0, 24.0) {
        state.checkbox1 = !state.checkbox1;
        state.last_click = frame;
        println!("Checkbox 1: {}", state.checkbox1);
        return;
    }
    if cursor.is_over(col1_px, cb_base_y + 35.0, 24.0, 24.0) {
        state.checkbox2 = !state.checkbox2;
        state.last_click = frame;
        println!("Checkbox 2: {}", state.checkbox2);
        return;
    }
    if cursor.is_over(col1_px, cb_base_y + 70.0, 24.0, 24.0) {
        state.checkbox3 = !state.checkbox3;
        state.last_click = frame;
        println!("Checkbox 3: {}", state.checkbox3);
        return;
    }

    // Switch positions
    let sw_base_y = cb_base_y + 105.0 + 20.0; // after checkboxes + label spacing
    if cursor.is_over(col1_px, sw_base_y, 44.0, 24.0) {
        state.switch1 = !state.switch1;
        state.last_click = frame;
        println!("Switch 1: {}", state.switch1);
        return;
    }
    if cursor.is_over(col1_px, sw_base_y + 35.0, 44.0, 24.0) {
        state.switch2 = !state.switch2;
        state.last_click = frame;
        println!("Switch 2: {}", state.switch2);
        return;
    }
    if cursor.is_over(col1_px, sw_base_y + 70.0, 44.0, 24.0) {
        state.dark_mode = !state.dark_mode;
        state.last_click = frame;
        println!("Dark mode: {}", state.dark_mode);
        return;
    }

    // Button positions (matching draw_buttons_info)
    let btn_base_y = start_y + 15.0 + 30.0; // card y + title spacing
    for i in 0..4 {
        if cursor.is_over(col3_px, btn_base_y + i as f32 * 50.0, 100.0, 40.0) {
            state.button_clicks[i] += 1;
            state.last_click = frame;
            println!("Button {} clicked: {}", i + 1, state.button_clicks[i]);
            return;
        }
    }

    // Open Dialog button
    let dialog_btn_y = btn_base_y + 200.0 + 10.0;
    if cursor.is_over(col3_px, dialog_btn_y, col_width - 30.0, 40.0) {
        state.show_dialog = true;
        state.last_click = frame;
        return;
    }

    // Dialog close button
    if state.show_dialog {
        let dialog_x = 350.0;
        let dialog_y = 260.0;
        // Cancel button
        if cursor.is_over(dialog_x + 20.0, dialog_y + 120.0, 80.0, 36.0) {
            state.show_dialog = false;
            state.last_click = frame;
            return;
        }
        // Confirm button
        if cursor.is_over(dialog_x + 200.0, dialog_y + 120.0, 80.0, 36.0) {
            state.show_dialog = false;
            state.last_click = frame;
            println!("Action confirmed!");
            return;
        }
    }
}


fn draw_header(canvas: &mut SoftwareBackend, theme: &Theme, width: f32) {
    canvas.draw_rect(Rect::new(0.0, 0.0, width, 60.0), &Paint::fill(theme.card));
    
    canvas.draw_text(
        "VenomUI Complete Widget Showcase",
        Point::new(20.0, 18.0),
        &Paint::fill(theme.text),
        22.0,
    );
    
    canvas.draw_text(
        "All widgets interactive • Click to interact",
        Point::new(20.0, 42.0),
        &Paint::fill(theme.muted),
        12.0,
    );
}

fn draw_form_controls(canvas: &mut SoftwareBackend, state: &AppState, theme: &Theme, cursor: Cursor, x: f32, y: f32, width: f32) {
    // Card background
    canvas.draw_rounded_rect(
        Rect::new(x, y, width, 320.0),
        BorderRadius::all(12.0),
        &Paint::fill(theme.card),
    );

    let px = x + 15.0;
    let mut py = y + 15.0;

    canvas.draw_text("Form Controls", Point::new(px, py), &Paint::fill(theme.text), 16.0);
    py += 30.0;

    // Checkboxes
    canvas.draw_text("Checkboxes", Point::new(px, py), &Paint::fill(theme.muted), 12.0);
    py += 20.0;

    draw_checkbox(canvas, px, py, state.checkbox1, "Enable notifications", theme, cursor);
    py += 35.0;
    draw_checkbox(canvas, px, py, state.checkbox2, "Auto-save drafts", theme, cursor);
    py += 35.0;
    draw_checkbox(canvas, px, py, state.checkbox3, "Show online status", theme, cursor);
    py += 50.0;

    // Switches
    canvas.draw_text("Switches", Point::new(px, py), &Paint::fill(theme.muted), 12.0);
    py += 20.0;

    draw_switch(canvas, px, py, state.switch1, "Wi-Fi", theme, cursor);
    py += 35.0;
    draw_switch(canvas, px, py, state.switch2, "Bluetooth", theme, cursor);
    py += 35.0;
    draw_switch(canvas, px, py, state.dark_mode, "Dark Mode", theme, cursor);
}

fn draw_sliders_progress(canvas: &mut SoftwareBackend, state: &AppState, theme: &Theme, cursor: Cursor, x: f32, y: f32, width: f32) {
    // Card background
    canvas.draw_rounded_rect(
        Rect::new(x, y, width, 320.0),
        BorderRadius::all(12.0),
        &Paint::fill(theme.card),
    );

    let px = x + 15.0;
    let mut py = y + 15.0;

    canvas.draw_text("Sliders & Progress", Point::new(px, py), &Paint::fill(theme.text), 16.0);
    py += 30.0;

    // Sliders
    canvas.draw_text("Volume", Point::new(px, py), &Paint::fill(theme.muted), 12.0);
    canvas.draw_text(&format!("{:.0}%", state.volume), Point::new(px + 230.0, py), &Paint::fill(theme.text), 12.0);
    py += 20.0;
    draw_slider(canvas, px, py, width - 30.0, state.volume / 100.0, theme.accent, state.dragging_slider == Some(0));
    py += 45.0;

    canvas.draw_text("Brightness", Point::new(px, py), &Paint::fill(theme.muted), 12.0);
    canvas.draw_text(&format!("{:.0}%", state.brightness), Point::new(px + 230.0, py), &Paint::fill(theme.text), 12.0);
    py += 20.0;
    draw_slider(canvas, px, py, width - 30.0, state.brightness / 100.0, theme.warning, state.dragging_slider == Some(1));
    py += 55.0;

    // Progress bars
    canvas.draw_text("Progress Indicators", Point::new(px, py), &Paint::fill(theme.muted), 12.0);
    py += 25.0;

    canvas.draw_text("Download", Point::new(px, py), &Paint::fill(theme.muted), 11.0);
    py += 18.0;
    draw_progress_bar(canvas, px, py, width - 30.0, state.download_progress, theme.accent);
    py += 25.0;

    canvas.draw_text("Upload", Point::new(px, py), &Paint::fill(theme.muted), 11.0);
    py += 18.0;
    draw_progress_bar(canvas, px, py, width - 30.0, state.upload_progress, theme.success);
    py += 35.0;

    // Progress circles
    draw_progress_circle(canvas, px + 40.0, py + 40.0, 35.0, state.download_progress, theme.accent);
    draw_progress_circle(canvas, px + 130.0, py + 40.0, 35.0, state.upload_progress, theme.success);
    draw_progress_circle(canvas, px + 220.0, py + 40.0, 35.0, 0.85, theme.warning);
}

fn draw_buttons_info(canvas: &mut SoftwareBackend, state: &AppState, theme: &Theme, cursor: Cursor, x: f32, y: f32, width: f32) {
    // Card background
    canvas.draw_rounded_rect(
        Rect::new(x, y, width, 320.0),
        BorderRadius::all(12.0),
        &Paint::fill(theme.card),
    );

    let px = x + 15.0;
    let mut py = y + 15.0;

    canvas.draw_text("Buttons", Point::new(px, py), &Paint::fill(theme.text), 16.0);
    py += 30.0;

    let colors = [theme.accent, theme.success, theme.warning, theme.danger];
    let labels = ["Primary", "Success", "Warning", "Danger"];

    for i in 0..4 {
        draw_button(canvas, px, py, 100.0, 40.0, labels[i], colors[i], cursor, state.button_clicks[i]);
        py += 50.0;
    }

    py += 10.0;
    draw_button(canvas, px, py, width - 30.0, 40.0, "Open Dialog", theme.accent, cursor, 0);

    // Stats
    py += 60.0;
    canvas.draw_text("Stats", Point::new(px, py), &Paint::fill(theme.muted), 12.0);
    py += 20.0;

    let total_clicks: u32 = state.button_clicks.iter().sum();
    canvas.draw_text(&format!("Total Clicks: {}", total_clicks), Point::new(px, py), &Paint::fill(theme.text), 11.0);
    py += 18.0;
    canvas.draw_text(&format!("Volume: {:.0}%", state.volume), Point::new(px, py), &Paint::fill(theme.text), 11.0);
    py += 18.0;
    canvas.draw_text(&format!("Brightness: {:.0}%", state.brightness), Point::new(px, py), &Paint::fill(theme.text), 11.0);
}

fn draw_dialog(canvas: &mut SoftwareBackend, state: &AppState, theme: &Theme, cursor: Cursor, width: f32, height: f32) {
    // Overlay
    canvas.draw_rect(Rect::new(0.0, 0.0, width, height), &Paint::fill(Color::rgba(0, 0, 0, 180)));

    // Dialog
    let dialog_w = 300.0;
    let dialog_h = 180.0;
    let dialog_x = (width - dialog_w) / 2.0;
    let dialog_y = (height - dialog_h) / 2.0;

    canvas.draw_rounded_rect(
        Rect::new(dialog_x, dialog_y, dialog_w, dialog_h),
        BorderRadius::all(16.0),
        &Paint::fill(theme.card),
    );

    canvas.draw_text("Confirm Action", Point::new(dialog_x + 20.0, dialog_y + 25.0), &Paint::fill(theme.text), 18.0);
    canvas.draw_text(
        "Are you sure you want to proceed?",
        Point::new(dialog_x + 20.0, dialog_y + 60.0),
        &Paint::fill(theme.muted),
        13.0,
    );
    canvas.draw_text(
        "This action cannot be undone.",
        Point::new(dialog_x + 20.0, dialog_y + 80.0),
        &Paint::fill(theme.muted),
        13.0,
    );

    // Buttons
    draw_button(canvas, dialog_x + 20.0, dialog_y + 120.0, 80.0, 36.0, "Cancel", Color::hex("#374151"), cursor, 0);
    draw_button(canvas, dialog_x + 200.0, dialog_y + 120.0, 80.0, 36.0, "Confirm", theme.accent, cursor, 0);
}

// ============================================================================
// WIDGET DRAWING HELPERS
// ============================================================================

fn draw_checkbox(canvas: &mut SoftwareBackend, x: f32, y: f32, checked: bool, label: &str, theme: &Theme, cursor: Cursor) {
    let hover = cursor.is_over(x, y, 24.0, 24.0);
    let color = if checked {
        if hover { Color::hex("#818cf8") } else { theme.accent }
    } else {
        if hover { Color::hex("#4b5563") } else { Color::hex("#374151") }
    };

    canvas.draw_rounded_rect(Rect::new(x, y, 24.0, 24.0), BorderRadius::all(4.0), &Paint::fill(color));

    if checked {
        let cx = x + 12.0;
        let cy = y + 12.0;
        canvas.draw_line(Point::new(cx - 5.0, cy), Point::new(cx - 1.0, cy + 4.0), &Paint::stroke(Color::WHITE, 2.5));
        canvas.draw_line(Point::new(cx - 1.0, cy + 4.0), Point::new(cx + 5.0, cy - 3.0), &Paint::stroke(Color::WHITE, 2.5));
    }

    canvas.draw_text(label, Point::new(x + 32.0, y + 4.0), &Paint::fill(theme.text), 13.0);
}

fn draw_switch(canvas: &mut SoftwareBackend, x: f32, y: f32, on: bool, label: &str, theme: &Theme, cursor: Cursor) {
    let hover = cursor.is_over(x, y, 44.0, 24.0);
    let track_color = if on {
        if hover { Color::hex("#4ade80") } else { theme.success }
    } else {
        if hover { Color::hex("#4b5563") } else { Color::hex("#374151") }
    };

    canvas.draw_rounded_rect(Rect::new(x, y, 44.0, 24.0), BorderRadius::all(12.0), &Paint::fill(track_color));

    let thumb_x = if on { x + 32.0 } else { x + 12.0 };
    canvas.draw_circle(Point::new(thumb_x, y + 12.0), 10.0, &Paint::fill(Color::WHITE));

    canvas.draw_text(label, Point::new(x + 52.0, y + 4.0), &Paint::fill(theme.text), 13.0);
}

fn draw_slider(canvas: &mut SoftwareBackend, x: f32, y: f32, width: f32, progress: f32, color: Color, dragging: bool) {
    canvas.draw_rounded_rect(Rect::new(x, y + 7.0, width, 6.0), BorderRadius::all(3.0), &Paint::fill(Color::hex("#374151")));
    
    let active_width = width * progress;
    if active_width > 0.0 {
        canvas.draw_rounded_rect(Rect::new(x, y + 7.0, active_width, 6.0), BorderRadius::all(3.0), &Paint::fill(color));
    }

    let thumb_x = x + active_width;
    let radius = if dragging { 11.0 } else { 9.0 };
    canvas.draw_circle(Point::new(thumb_x, y + 10.0), radius, &Paint::fill(Color::WHITE));
    canvas.draw_circle(Point::new(thumb_x, y + 10.0), radius, &Paint::stroke(color, 2.0));
}

fn draw_progress_bar(canvas: &mut SoftwareBackend, x: f32, y: f32, width: f32, progress: f32, color: Color) {
    canvas.draw_rounded_rect(Rect::new(x, y, width, 8.0), BorderRadius::all(4.0), &Paint::fill(Color::hex("#374151")));
    
    let active_width = width * progress.clamp(0.0, 1.0);
    if active_width > 0.0 {
        canvas.draw_rounded_rect(Rect::new(x, y, active_width, 8.0), BorderRadius::all(4.0), &Paint::fill(color));
    }
}

fn draw_progress_circle(canvas: &mut SoftwareBackend, cx: f32, cy: f32, radius: f32, progress: f32, color: Color) {
    canvas.draw_circle(Point::new(cx, cy), radius, &Paint::stroke(Color::hex("#374151"), 6.0));
    
    // Draw arc (simplified - just show percentage text)
    canvas.draw_circle(Point::new(cx, cy), radius - 8.0, &Paint::fill(color.with_alpha(50)));
    
    let text = format!("{:.0}%", progress * 100.0);
    canvas.draw_text(&text, Point::new(cx - 12.0, cy - 5.0), &Paint::fill(Color::WHITE), 11.0);
}

fn draw_button(canvas: &mut SoftwareBackend, x: f32, y: f32, w: f32, h: f32, label: &str, color: Color, cursor: Cursor, count: u32) {
    let hover = cursor.is_over(x, y, w, h);
    let color = if hover {
        Color::rgba((color.r as u16 + 25).min(255) as u8, (color.g as u16 + 25).min(255) as u8, (color.b as u16 + 25).min(255) as u8, 255)
    } else {
        color
    };

    canvas.draw_rounded_rect(Rect::new(x, y, w, h), BorderRadius::all(8.0), &Paint::fill(color));

    let text_x = x + (w - label.len() as f32 * 7.0) / 2.0;
    canvas.draw_text(label, Point::new(text_x, y + h / 2.0 - 6.0), &Paint::fill(Color::WHITE), 13.0);

    if count > 0 {
        canvas.draw_text(&format!("×{}", count), Point::new(x + w - 25.0, y + h / 2.0 - 5.0), &Paint::fill(Color::rgba(255, 255, 255, 180)), 10.0);
    }
}
