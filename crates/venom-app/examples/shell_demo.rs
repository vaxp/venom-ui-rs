//! VenomUI Shell Demo - Iced-style event handling
//!
//! Demonstrates the Shell pattern for message passing and event capture.
//!
//! Run with: cargo run --example shell_demo

use venom_app::{App, AppConfig, RenderContext};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point, BorderRadius};
use venom_widgets::{Shell, EventStatus, Cursor};

// ============================================================================
// APPLICATION MESSAGE
// ============================================================================

/// Messages that widgets can send to the application
#[derive(Debug, Clone)]
enum Message {
    IncrementCounter,
    DecrementCounter,
    ToggleTheme,
    SliderChanged(f32),
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

struct AppState {
    counter: i32,
    dark_theme: bool,
    slider_value: f32,
    messages: Vec<Message>,
    
    // Widget bounds for hit testing
    inc_button: (f32, f32, f32, f32),
    dec_button: (f32, f32, f32, f32),
    theme_button: (f32, f32, f32, f32),
    slider_bounds: (f32, f32, f32, f32),
    
    // Interaction state
    slider_dragging: bool,
    last_click_frame: u64,
}

impl AppState {
    fn new() -> Self {
        Self {
            counter: 0,
            dark_theme: true,
            slider_value: 50.0,
            messages: Vec::new(),
            
            inc_button: (50.0, 150.0, 100.0, 40.0),
            dec_button: (170.0, 150.0, 100.0, 40.0),
            theme_button: (50.0, 210.0, 220.0, 40.0),
            slider_bounds: (50.0, 280.0, 220.0, 30.0),
            
            slider_dragging: false,
            last_click_frame: 0,
        }
    }

    fn process_messages(&mut self) {
        for msg in self.messages.drain(..) {
            match msg {
                Message::IncrementCounter => {
                    self.counter += 1;
                    println!("📈 Counter incremented to {}", self.counter);
                }
                Message::DecrementCounter => {
                    self.counter -= 1;
                    println!("📉 Counter decremented to {}", self.counter);
                }
                Message::ToggleTheme => {
                    self.dark_theme = !self.dark_theme;
                    println!("🎨 Theme toggled: {}", if self.dark_theme { "Dark" } else { "Light" });
                }
                Message::SliderChanged(value) => {
                    self.slider_value = value;
                }
            }
        }
    }

    fn handle_click(&mut self, cursor: Cursor, frame: u64) {
        // Debounce - only one click per 10 frames
        if frame - self.last_click_frame < 10 {
            return;
        }

        let mut shell = Shell::new(&mut self.messages);

        // Check increment button
        if cursor.is_over(self.inc_button.0, self.inc_button.1, self.inc_button.2, self.inc_button.3) {
            shell.publish(Message::IncrementCounter);
            shell.capture_event();
            self.last_click_frame = frame;
        }

        // Check decrement button (only if not captured)
        if !shell.is_event_captured() && 
           cursor.is_over(self.dec_button.0, self.dec_button.1, self.dec_button.2, self.dec_button.3) {
            shell.publish(Message::DecrementCounter);
            shell.capture_event();
            self.last_click_frame = frame;
        }

        // Check theme button
        if !shell.is_event_captured() && 
           cursor.is_over(self.theme_button.0, self.theme_button.1, self.theme_button.2, self.theme_button.3) {
            shell.publish(Message::ToggleTheme);
            shell.capture_event();
            self.last_click_frame = frame;
        }

        // Check slider
        if !shell.is_event_captured() &&
           cursor.is_over(self.slider_bounds.0, self.slider_bounds.1, self.slider_bounds.2, self.slider_bounds.3) {
            self.slider_dragging = true;
            if let Some((x, _)) = cursor.position {
                let value = ((x - self.slider_bounds.0) / self.slider_bounds.2 * 100.0).clamp(0.0, 100.0);
                shell.publish(Message::SliderChanged(value));
            }
            shell.capture_event();
        }

        // Log event status
        if shell.is_event_captured() {
            println!("🎯 Event captured by widget");
        }
    }

    fn handle_drag(&mut self, cursor: Cursor) {
        if self.slider_dragging {
            if let Some((x, _)) = cursor.position {
                let value = ((x - self.slider_bounds.0) / self.slider_bounds.2 * 100.0).clamp(0.0, 100.0);
                self.slider_value = value;
            }
        }
    }

    fn handle_release(&mut self) {
        if self.slider_dragging {
            self.slider_dragging = false;
            println!("📊 Slider value: {:.1}%", self.slider_value);
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
    println!("🐚 VenomUI Shell Demo");
    println!("======================");
    println!("Demonstrates Iced-style event handling with Shell\n");
    
    App::run(
        AppConfig::new("VenomUI Shell Demo")
            .size(600, 400)
            .background(Color::hex("#0f172a")),
        render,
    );
}

fn render(canvas: &mut SoftwareBackend, ctx: &RenderContext) {
    let state = get_state();

    // Create cursor from context
    let cursor = match ctx.mouse_position {
        Some(pos) => Cursor::at(pos.x, pos.y),
        None => Cursor::unavailable(),
    };

    // Handle input
    if ctx.mouse_pressed {
        state.handle_click(cursor, ctx.frame);
        state.handle_drag(cursor);
    } else {
        state.handle_release();
    }

    // If dragging, continue tracking
    if state.slider_dragging && ctx.mouse_pressed {
        state.handle_drag(cursor);
    }

    // Process all messages
    state.process_messages();

    // Get theme colors
    let (bg, text, accent) = if state.dark_theme {
        (Color::hex("#0f172a"), Color::WHITE, Color::hex("#6366f1"))
    } else {
        (Color::hex("#f8fafc"), Color::hex("#1e293b"), Color::hex("#4f46e5"))
    };

    // Draw background
    canvas.draw_rect(Rect::new(0.0, 0.0, ctx.width as f32, ctx.height as f32), &Paint::fill(bg));

    // Title
    canvas.draw_text(
        "Shell Event Demo",
        Point::new(50.0, 40.0),
        &Paint::fill(text),
        28.0,
    );

    canvas.draw_text(
        "Events bubble up until captured by Shell.capture_event()",
        Point::new(50.0, 75.0),
        &Paint::fill(Color::hex("#64748b")),
        14.0,
    );

    // Counter display
    let counter_text = format!("Counter: {}", state.counter);
    canvas.draw_rounded_rect(
        Rect::new(50.0, 100.0, 220.0, 40.0),
        BorderRadius::all(8.0),
        &Paint::fill(if state.dark_theme { Color::hex("#1e293b") } else { Color::hex("#e2e8f0") }),
    );
    canvas.draw_text(
        &counter_text,
        Point::new(110.0, 112.0),
        &Paint::fill(text),
        18.0,
    );

    // Increment button
    draw_button(canvas, state.inc_button, "+", accent, cursor);

    // Decrement button
    draw_button(canvas, state.dec_button, "-", Color::hex("#ef4444"), cursor);

    // Theme toggle button
    let theme_label = if state.dark_theme { "☀️ Light Mode" } else { "🌙 Dark Mode" };
    draw_button(canvas, state.theme_button, theme_label, Color::hex("#22c55e"), cursor);

    // Slider
    draw_slider(canvas, state.slider_bounds, state.slider_value, accent, cursor, state.slider_dragging);

    // Slider value
    canvas.draw_text(
        &format!("Value: {:.0}%", state.slider_value),
        Point::new(290.0, 287.0),
        &Paint::fill(text),
        14.0,
    );

    // Info panel
    let panel_x = 380.0;
    canvas.draw_rounded_rect(
        Rect::new(panel_x, 100.0, 180.0, 180.0),
        BorderRadius::all(12.0),
        &Paint::fill(if state.dark_theme { Color::hex("#1e293b") } else { Color::hex("#e2e8f0") }),
    );

    canvas.draw_text("Shell Info", Point::new(panel_x + 15.0, 120.0), &Paint::fill(text), 16.0);
    
    let info = [
        format!("Counter: {}", state.counter),
        format!("Slider: {:.0}%", state.slider_value),
        format!("Theme: {}", if state.dark_theme { "Dark" } else { "Light" }),
        format!("Frame: {}", ctx.frame),
    ];

    for (i, line) in info.iter().enumerate() {
        canvas.draw_text(
            line,
            Point::new(panel_x + 15.0, 150.0 + i as f32 * 24.0),
            &Paint::fill(Color::hex("#64748b")),
            12.0,
        );
    }

    // Mouse cursor indicator
    if let Some(pos) = ctx.mouse_position {
        canvas.draw_circle(pos, 6.0, &Paint::fill(Color::rgba(99, 102, 241, 200)));
        canvas.draw_circle(pos, 6.0, &Paint::stroke(Color::WHITE, 1.5));
    }
}

fn draw_button(canvas: &mut SoftwareBackend, bounds: (f32, f32, f32, f32), label: &str, color: Color, cursor: Cursor) {
    let (x, y, w, h) = bounds;
    let hover = cursor.is_over(x, y, w, h);
    
    let color = if hover {
        Color::rgba(
            (color.r as u16 + 30).min(255) as u8,
            (color.g as u16 + 30).min(255) as u8,
            (color.b as u16 + 30).min(255) as u8,
            255,
        )
    } else {
        color
    };

    canvas.draw_rounded_rect(
        Rect::new(x, y, w, h),
        BorderRadius::all(8.0),
        &Paint::fill(color),
    );

    let text_x = x + (w - label.chars().count() as f32 * 7.0) / 2.0;
    canvas.draw_text(
        label,
        Point::new(text_x, y + 12.0),
        &Paint::fill(Color::WHITE),
        14.0,
    );
}

fn draw_slider(canvas: &mut SoftwareBackend, bounds: (f32, f32, f32, f32), value: f32, color: Color, cursor: Cursor, dragging: bool) {
    let (x, y, w, h) = bounds;
    let track_height = 6.0;
    let track_y = y + (h - track_height) / 2.0;

    // Track background
    canvas.draw_rounded_rect(
        Rect::new(x, track_y, w, track_height),
        BorderRadius::all(3.0),
        &Paint::fill(Color::hex("#374151")),
    );

    // Active track
    let progress = value / 100.0;
    let active_width = w * progress;
    if active_width > 0.0 {
        canvas.draw_rounded_rect(
            Rect::new(x, track_y, active_width, track_height),
            BorderRadius::all(3.0),
            &Paint::fill(color),
        );
    }

    // Thumb
    let thumb_x = x + active_width;
    let thumb_y = y + h / 2.0;
    let thumb_radius = if dragging { 12.0 } else { 10.0 };

    canvas.draw_circle(
        Point::new(thumb_x, thumb_y),
        thumb_radius,
        &Paint::fill(Color::WHITE),
    );
    canvas.draw_circle(
        Point::new(thumb_x, thumb_y),
        thumb_radius,
        &Paint::stroke(color, 2.0),
    );
}
