//! VenomUI Interactive Demo - Widgets with click handling
//!
//! Run with: cargo run --example interactive_demo

use venom_app::{App, AppConfig, RenderContext};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point, BorderRadius, Offset};
use venom_widgets::{
    Widget, WidgetBounds, InteractionState,
};

// ============================================================================
// INTERACTIVE CHECKBOX STATE
// ============================================================================

struct CheckboxState {
    checked: bool,
    bounds: WidgetBounds,
    hover: bool,
}

impl CheckboxState {
    fn new(x: f32, y: f32, checked: bool) -> Self {
        Self {
            checked,
            bounds: WidgetBounds::from_xywh(x, y, 24.0, 24.0),
            hover: false,
        }
    }

    fn toggle(&mut self) {
        self.checked = !self.checked;
        println!("Checkbox toggled: {}", self.checked);
    }
}

// ============================================================================
// INTERACTIVE SWITCH STATE
// ============================================================================

struct SwitchState {
    on: bool,
    bounds: WidgetBounds,
    hover: bool,
}

impl SwitchState {
    fn new(x: f32, y: f32, on: bool) -> Self {
        Self {
            on,
            bounds: WidgetBounds::from_xywh(x, y, 44.0, 24.0),
            hover: false,
        }
    }

    fn toggle(&mut self) {
        self.on = !self.on;
        println!("Switch toggled: {}", self.on);
    }
}

// ============================================================================
// INTERACTIVE BUTTON STATE
// ============================================================================

struct ButtonState {
    label: &'static str,
    bounds: WidgetBounds,
    color: Color,
    hover: bool,
    pressed: bool,
    click_count: u32,
}

impl ButtonState {
    fn new(x: f32, y: f32, label: &'static str, color: Color) -> Self {
        Self {
            label,
            bounds: WidgetBounds::from_xywh(x, y, 100.0, 40.0),
            color,
            hover: false,
            pressed: false,
            click_count: 0,
        }
    }

    fn click(&mut self) {
        self.click_count += 1;
        println!("Button '{}' clicked! (count: {})", self.label, self.click_count);
    }
}

// ============================================================================
// INTERACTIVE SLIDER STATE
// ============================================================================

struct SliderState {
    value: f32,
    min: f32,
    max: f32,
    bounds: WidgetBounds,
    dragging: bool,
}

impl SliderState {
    fn new(x: f32, y: f32, value: f32) -> Self {
        Self {
            value,
            min: 0.0,
            max: 100.0,
            bounds: WidgetBounds::from_xywh(x, y, 200.0, 20.0),
            dragging: false,
        }
    }

    fn set_from_x(&mut self, mouse_x: f32) {
        let relative = (mouse_x - self.bounds.offset.dx) / self.bounds.size.width;
        self.value = self.min + (self.max - self.min) * relative.clamp(0.0, 1.0);
        println!("Slider value: {:.1}", self.value);
    }

    fn progress(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }
}

// ============================================================================
// APP STATE
// ============================================================================

struct AppState {
    checkboxes: Vec<CheckboxState>,
    switches: Vec<SwitchState>,
    buttons: Vec<ButtonState>,
    sliders: Vec<SliderState>,
    total_clicks: u32,
}

impl AppState {
    fn new() -> Self {
        let padding = 40.0;
        
        Self {
            checkboxes: vec![
                CheckboxState::new(padding, 100.0, false),
                CheckboxState::new(padding + 150.0, 100.0, true),
                CheckboxState::new(padding + 300.0, 100.0, false),
            ],
            switches: vec![
                SwitchState::new(padding, 180.0, false),
                SwitchState::new(padding + 150.0, 180.0, true),
            ],
            buttons: vec![
                ButtonState::new(padding, 260.0, "Primary", Color::hex("#6366f1")),
                ButtonState::new(padding + 120.0, 260.0, "Success", Color::hex("#22c55e")),
                ButtonState::new(padding + 240.0, 260.0, "Danger", Color::hex("#ef4444")),
            ],
            sliders: vec![
                SliderState::new(padding, 350.0, 50.0),
                SliderState::new(padding, 400.0, 25.0),
            ],
            total_clicks: 0,
        }
    }

    fn handle_click(&mut self, mouse: Point) {
        // Check checkboxes
        for cb in &mut self.checkboxes {
            if cb.bounds.contains(mouse) {
                cb.toggle();
                self.total_clicks += 1;
                return;
            }
        }

        // Check switches
        for sw in &mut self.switches {
            if sw.bounds.contains(mouse) {
                sw.toggle();
                self.total_clicks += 1;
                return;
            }
        }

        // Check buttons
        for btn in &mut self.buttons {
            if btn.bounds.contains(mouse) {
                btn.click();
                self.total_clicks += 1;
                return;
            }
        }

        // Check sliders (start drag)
        for slider in &mut self.sliders {
            if slider.bounds.contains(mouse) {
                slider.dragging = true;
                slider.set_from_x(mouse.x);
                return;
            }
        }
    }

    fn handle_release(&mut self) {
        for slider in &mut self.sliders {
            slider.dragging = false;
        }
    }

    fn handle_move(&mut self, mouse: Point) {
        // Update hover states
        for cb in &mut self.checkboxes {
            cb.hover = cb.bounds.contains(mouse);
        }
        for sw in &mut self.switches {
            sw.hover = sw.bounds.contains(mouse);
        }
        for btn in &mut self.buttons {
            btn.hover = btn.bounds.contains(mouse);
        }

        // Handle slider dragging
        for slider in &mut self.sliders {
            if slider.dragging {
                slider.set_from_x(mouse.x);
            }
        }
    }
}

// Global mutable state (for demo purposes)
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
    println!("🎮 VenomUI Interactive Demo");
    println!("===========================");
    println!("Click on widgets to interact!\n");
    
    App::run(
        AppConfig::new("VenomUI Interactive Demo")
            .size(800, 500)
            .background(Color::hex("#0f172a")),
        render,
    );
}

fn render(canvas: &mut SoftwareBackend, ctx: &RenderContext) {
    let state = get_state();
    let padding = 40.0;

    // Handle mouse events
    if let Some(mouse) = ctx.mouse_position {
        state.handle_move(mouse);
        
        // Check for click (frame 0 = just pressed)
        if ctx.mouse_pressed {
            state.handle_click(mouse);
        }
    }

    if !ctx.mouse_pressed {
        state.handle_release();
    }

    // Title
    canvas.draw_text(
        "Interactive Widget Demo",
        Point::new(padding, 30.0),
        &Paint::fill(Color::WHITE),
        24.0,
    );

    canvas.draw_text(
        "Click on widgets to interact!",
        Point::new(padding, 55.0),
        &Paint::fill(Color::hex("#9ca3af")),
        14.0,
    );

    // ========================================================================
    // CHECKBOXES
    // ========================================================================
    draw_section_header(canvas, "Checkboxes (click to toggle)", padding, 80.0);
    
    for (i, cb) in state.checkboxes.iter().enumerate() {
        draw_checkbox(canvas, cb);
        let label = if cb.checked { "Checked" } else { "Unchecked" };
        canvas.draw_text(
            label,
            Point::new(cb.bounds.offset.dx + 32.0, cb.bounds.offset.dy + 4.0),
            &Paint::fill(Color::hex("#9ca3af")),
            14.0,
        );
    }

    // ========================================================================
    // SWITCHES
    // ========================================================================
    draw_section_header(canvas, "Switches (click to toggle)", padding, 160.0);
    
    for sw in &state.switches {
        draw_switch(canvas, sw);
        let label = if sw.on { "ON" } else { "OFF" };
        canvas.draw_text(
            label,
            Point::new(sw.bounds.offset.dx + 52.0, sw.bounds.offset.dy + 4.0),
            &Paint::fill(if sw.on { Color::hex("#22c55e") } else { Color::hex("#6b7280") }),
            14.0,
        );
    }

    // ========================================================================
    // BUTTONS
    // ========================================================================
    draw_section_header(canvas, "Buttons (click to increment)", padding, 240.0);
    
    for btn in &state.buttons {
        draw_button(canvas, btn);
    }

    // ========================================================================
    // SLIDERS
    // ========================================================================
    draw_section_header(canvas, "Sliders (click and drag)", padding, 330.0);
    
    for slider in &state.sliders {
        draw_slider(canvas, slider);
        canvas.draw_text(
            &format!("{:.0}%", slider.value),
            Point::new(slider.bounds.offset.dx + 210.0, slider.bounds.offset.dy),
            &Paint::fill(Color::WHITE),
            14.0,
        );
    }

    // ========================================================================
    // STATS PANEL
    // ========================================================================
    let panel_x = 550.0;
    canvas.draw_rounded_rect(
        Rect::new(panel_x, 80.0, 210.0, 150.0),
        BorderRadius::all(12.0),
        &Paint::fill(Color::hex("#1e293b")),
    );

    canvas.draw_text("Stats", Point::new(panel_x + 20.0, 100.0), 
        &Paint::fill(Color::WHITE), 16.0);

    canvas.draw_text(
        &format!("Total Clicks: {}", state.total_clicks),
        Point::new(panel_x + 20.0, 130.0),
        &Paint::fill(Color::hex("#9ca3af")),
        14.0,
    );

    let checked_count = state.checkboxes.iter().filter(|c| c.checked).count();
    canvas.draw_text(
        &format!("Checked: {}/3", checked_count),
        Point::new(panel_x + 20.0, 155.0),
        &Paint::fill(Color::hex("#9ca3af")),
        14.0,
    );

    let on_count = state.switches.iter().filter(|s| s.on).count();
    canvas.draw_text(
        &format!("Switches On: {}/2", on_count),
        Point::new(panel_x + 20.0, 180.0),
        &Paint::fill(Color::hex("#9ca3af")),
        14.0,
    );

    // Mouse cursor
    if let Some(mouse) = ctx.mouse_position {
        canvas.draw_circle(mouse, 6.0, &Paint::fill(Color::rgba(99, 102, 241, 200)));
        canvas.draw_circle(mouse, 6.0, &Paint::stroke(Color::WHITE, 1.5));
    }
}

fn draw_section_header(canvas: &mut SoftwareBackend, title: &str, x: f32, y: f32) {
    canvas.draw_text(
        title,
        Point::new(x, y),
        &Paint::fill(Color::hex("#e2e8f0")),
        14.0,
    );
}

fn draw_checkbox(canvas: &mut SoftwareBackend, cb: &CheckboxState) {
    let b = &cb.bounds;
    let size = b.size.width;
    
    let bg_color = if cb.hover {
        if cb.checked { Color::hex("#818cf8") } else { Color::hex("#4b5563") }
    } else {
        if cb.checked { Color::hex("#6366f1") } else { Color::hex("#374151") }
    };

    canvas.draw_rounded_rect(
        b.rect(),
        BorderRadius::all(4.0),
        &Paint::fill(bg_color),
    );

    if cb.checked {
        // Draw checkmark
        let cx = b.offset.dx + size / 2.0;
        let cy = b.offset.dy + size / 2.0;
        let s = size * 0.25;

        canvas.draw_line(
            Point::new(cx - s, cy),
            Point::new(cx - s * 0.3, cy + s * 0.7),
            &Paint::stroke(Color::WHITE, 2.5),
        );
        canvas.draw_line(
            Point::new(cx - s * 0.3, cy + s * 0.7),
            Point::new(cx + s, cy - s * 0.5),
            &Paint::stroke(Color::WHITE, 2.5),
        );
    }
}

fn draw_switch(canvas: &mut SoftwareBackend, sw: &SwitchState) {
    let b = &sw.bounds;
    let height = b.size.height;
    let radius = height / 2.0;

    let track_color = if sw.on {
        if sw.hover { Color::hex("#4ade80") } else { Color::hex("#22c55e") }
    } else {
        if sw.hover { Color::hex("#4b5563") } else { Color::hex("#374151") }
    };

    canvas.draw_rounded_rect(b.rect(), BorderRadius::all(radius), &Paint::fill(track_color));

    let thumb_x = if sw.on {
        b.offset.dx + b.size.width - radius
    } else {
        b.offset.dx + radius
    };

    canvas.draw_circle(
        Point::new(thumb_x, b.offset.dy + radius),
        radius - 2.0,
        &Paint::fill(Color::WHITE),
    );
}

fn draw_button(canvas: &mut SoftwareBackend, btn: &ButtonState) {
    let b = &btn.bounds;
    
    let color = if btn.hover {
        // Lighten on hover
        Color::rgba(
            (btn.color.r as u16 + 30).min(255) as u8,
            (btn.color.g as u16 + 30).min(255) as u8,
            (btn.color.b as u16 + 30).min(255) as u8,
            255,
        )
    } else {
        btn.color
    };

    canvas.draw_rounded_rect(b.rect(), BorderRadius::all(8.0), &Paint::fill(color));

    let text_x = b.offset.dx + (b.size.width - btn.label.len() as f32 * 7.0) / 2.0;
    canvas.draw_text(
        btn.label,
        Point::new(text_x, b.offset.dy + 12.0),
        &Paint::fill(Color::WHITE),
        14.0,
    );

    // Show click count
    if btn.click_count > 0 {
        canvas.draw_text(
            &format!("×{}", btn.click_count),
            Point::new(b.offset.dx + b.size.width - 25.0, b.offset.dy + 12.0),
            &Paint::fill(Color::rgba(255, 255, 255, 180)),
            11.0,
        );
    }
}

fn draw_slider(canvas: &mut SoftwareBackend, slider: &SliderState) {
    let b = &slider.bounds;
    let track_height = 6.0;
    let track_y = b.offset.dy + (b.size.height - track_height) / 2.0;

    // Track background
    canvas.draw_rounded_rect(
        Rect::new(b.offset.dx, track_y, b.size.width, track_height),
        BorderRadius::all(3.0),
        &Paint::fill(Color::hex("#374151")),
    );

    // Active track
    let active_width = b.size.width * slider.progress();
    if active_width > 0.0 {
        canvas.draw_rounded_rect(
            Rect::new(b.offset.dx, track_y, active_width, track_height),
            BorderRadius::all(3.0),
            &Paint::fill(Color::hex("#6366f1")),
        );
    }

    // Thumb
    let thumb_x = b.offset.dx + active_width;
    let thumb_y = b.offset.dy + b.size.height / 2.0;
    let thumb_radius = if slider.dragging { 10.0 } else { 8.0 };

    canvas.draw_circle(
        Point::new(thumb_x, thumb_y),
        thumb_radius,
        &Paint::fill(Color::WHITE),
    );
    canvas.draw_circle(
        Point::new(thumb_x, thumb_y),
        thumb_radius,
        &Paint::stroke(Color::hex("#6366f1"), 2.0),
    );
}
