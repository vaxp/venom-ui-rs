//! VenomUI Widgets Demo - Showcase all widgets
//!
//! Run with: cargo run --example widgets_demo

use venom_app::{App, AppConfig, RenderContext};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point, BorderRadius, Offset, BoxConstraints};
use venom_widgets::{
    Widget, Checkbox, Switch, Slider, ProgressBar, ProgressCircle,
    TextField, Text, Button,
};

fn main() {
    println!("🎨 VenomUI Widgets Demo");
    println!("========================");
    println!("Testing all form widgets...\n");
    
    App::run(
        AppConfig::new("VenomUI Widgets Demo")
            .size(900, 700)
            .background(Color::hex("#0f172a")),
        render,
    );
}

fn render(canvas: &mut SoftwareBackend, ctx: &RenderContext) {
    let padding = 30.0;
    let time = ctx.frame as f32 * 0.03;
    
    // Title
    canvas.draw_text(
        "VenomUI Widgets Demo",
        Point::new(padding, padding),
        &Paint::fill(Color::WHITE),
        24.0,
    );
    
    canvas.draw_text(
        "Testing form controls and progress indicators",
        Point::new(padding, padding + 30.0),
        &Paint::fill(Color::hex("#9ca3af")),
        14.0,
    );

    // ========================================================================
    // SECTION 1: Checkboxes
    // ========================================================================
    let section_y = 100.0;
    draw_section_header(canvas, "Checkboxes", padding, section_y);
    
    // Checkbox 1 - Unchecked
    let cb1 = Checkbox::new(false).label("Unchecked");
    cb1.paint(canvas, Offset::new(padding, section_y + 30.0));
    
    // Checkbox 2 - Checked
    let cb2 = Checkbox::checked().label("Checked");
    cb2.paint(canvas, Offset::new(padding + 150.0, section_y + 30.0));
    
    // Checkbox 3 - Disabled
    let cb3 = Checkbox::new(true).label("Disabled").disabled();
    cb3.paint(canvas, Offset::new(padding + 300.0, section_y + 30.0));

    // ========================================================================
    // SECTION 2: Switches
    // ========================================================================
    let section_y = 180.0;
    draw_section_header(canvas, "Switches", padding, section_y);
    
    // Switch states based on animation
    let switch_on = (time.sin() > 0.0);
    
    let sw1 = Switch::off();
    sw1.paint(canvas, Offset::new(padding, section_y + 30.0));
    
    canvas.draw_text("Off", Point::new(padding + 50.0, section_y + 34.0), 
        &Paint::fill(Color::hex("#9ca3af")), 14.0);
    
    let sw2 = Switch::on();
    sw2.paint(canvas, Offset::new(padding + 120.0, section_y + 30.0));
    
    canvas.draw_text("On", Point::new(padding + 170.0, section_y + 34.0), 
        &Paint::fill(Color::hex("#22c55e")), 14.0);
    
    let sw3 = Switch::new(false).disabled();
    sw3.paint(canvas, Offset::new(padding + 240.0, section_y + 30.0));
    
    canvas.draw_text("Disabled", Point::new(padding + 290.0, section_y + 34.0), 
        &Paint::fill(Color::hex("#6b7280")), 14.0);

    // ========================================================================
    // SECTION 3: Sliders
    // ========================================================================
    let section_y = 260.0;
    draw_section_header(canvas, "Sliders", padding, section_y);
    
    // Animated slider value
    let slider_value = ((time.sin() + 1.0) / 2.0) * 100.0;
    
    let slider1 = Slider::new(0.0..=100.0, slider_value).width(250.0);
    slider1.paint(canvas, Offset::new(padding, section_y + 35.0));
    
    canvas.draw_text(
        &format!("Value: {:.0}%", slider_value),
        Point::new(padding + 270.0, section_y + 35.0),
        &Paint::fill(Color::hex("#9ca3af")),
        14.0,
    );
    
    // Second slider with different color
    let slider2 = Slider::new(0.0..=100.0, 75.0)
        .width(250.0)
        .active_color(Color::hex("#22c55e"));
    slider2.paint(canvas, Offset::new(padding, section_y + 70.0));

    // ========================================================================
    // SECTION 4: Progress Indicators
    // ========================================================================
    let section_y = 360.0;
    draw_section_header(canvas, "Progress Indicators", padding, section_y);
    
    // Animated progress
    let progress = ((time.sin() + 1.0) / 2.0).clamp(0.0, 1.0);
    
    // Progress bars
    let pb1 = ProgressBar::new(progress).width(200.0).height(8.0);
    pb1.paint(canvas, Offset::new(padding, section_y + 35.0));
    
    canvas.draw_text(
        &format!("{:.0}%", progress * 100.0),
        Point::new(padding + 220.0, section_y + 32.0),
        &Paint::fill(Color::WHITE),
        14.0,
    );
    
    let pb2 = ProgressBar::new(0.6)
        .width(200.0)
        .height(8.0)
        .color(Color::hex("#22c55e"));
    pb2.paint(canvas, Offset::new(padding, section_y + 60.0));
    
    let pb3 = ProgressBar::new(0.3)
        .width(200.0)
        .height(8.0)
        .color(Color::hex("#f97316"));
    pb3.paint(canvas, Offset::new(padding, section_y + 85.0));
    
    // Progress circles
    let circle_x = padding + 350.0;
    
    let pc1 = ProgressCircle::new(progress).size(50.0);
    pc1.paint(canvas, Offset::new(circle_x, section_y + 30.0));
    
    let pc2 = ProgressCircle::new(0.75)
        .size(50.0)
        .color(Color::hex("#22c55e"));
    pc2.paint(canvas, Offset::new(circle_x + 70.0, section_y + 30.0));

    // ========================================================================
    // SECTION 5: Text Fields
    // ========================================================================
    let section_y = 480.0;
    draw_section_header(canvas, "Text Fields", padding, section_y);
    
    let tf1 = TextField::new("Hello VenomUI!")
        .width(250.0);
    tf1.paint(canvas, Offset::new(padding, section_y + 30.0));
    
    let tf2 = TextField::new("")
        .placeholder("Enter your email...")
        .width(250.0);
    tf2.paint(canvas, Offset::new(padding + 280.0, section_y + 30.0));
    
    let tf3 = TextField::new("secret123")
        .placeholder("Password")
        .obscure_text(true)
        .width(200.0);
    tf3.paint(canvas, Offset::new(padding + 560.0, section_y + 30.0));

    // ========================================================================
    // SECTION 6: Buttons
    // ========================================================================
    let section_y = 570.0;
    draw_section_header(canvas, "Buttons", padding, section_y);
    
    // Draw buttons manually (since Button needs interaction handling)
    draw_button(canvas, "Primary", padding, section_y + 30.0, Color::hex("#6366f1"));
    draw_button(canvas, "Success", padding + 120.0, section_y + 30.0, Color::hex("#22c55e"));
    draw_button(canvas, "Warning", padding + 240.0, section_y + 30.0, Color::hex("#f97316"));
    draw_button(canvas, "Danger", padding + 360.0, section_y + 30.0, Color::hex("#ef4444"));

    // ========================================================================
    // Right Panel - Widget Info
    // ========================================================================
    let panel_x = ctx.width as f32 - 250.0;
    
    canvas.draw_rounded_rect(
        Rect::new(panel_x, 100.0, 220.0, 400.0),
        BorderRadius::all(12.0),
        &Paint::fill(Color::hex("#1e293b")),
    );
    
    canvas.draw_text("Widget Info", Point::new(panel_x + 20.0, 120.0), 
        &Paint::fill(Color::WHITE), 16.0);
    
    let info_lines = [
        ("Checkboxes", "3"),
        ("Switches", "3"),
        ("Sliders", "2"),
        ("Progress Bars", "3"),
        ("Progress Circles", "2"),
        ("Text Fields", "3"),
        ("Buttons", "4"),
        ("", ""),
        ("Total Widgets", "20"),
    ];
    
    let mut y = 160.0;
    for (label, value) in info_lines {
        if label.is_empty() {
            y += 10.0;
            continue;
        }
        canvas.draw_text(label, Point::new(panel_x + 20.0, y), 
            &Paint::fill(Color::hex("#9ca3af")), 12.0);
        canvas.draw_text(value, Point::new(panel_x + 180.0, y), 
            &Paint::fill(Color::WHITE), 12.0);
        y += 24.0;
    }
    
    // Mouse position indicator
    if let Some(mouse) = ctx.mouse_position {
        canvas.draw_circle(mouse, 8.0, &Paint::fill(Color::rgba(99, 102, 241, 150)));
        canvas.draw_circle(mouse, 8.0, &Paint::stroke(Color::WHITE, 2.0));
    }
    
    // Frame counter
    canvas.draw_text(
        &format!("Frame: {}", ctx.frame),
        Point::new(panel_x + 20.0, 480.0),
        &Paint::fill(Color::hex("#6b7280")),
        11.0,
    );
}

fn draw_section_header(canvas: &mut SoftwareBackend, title: &str, x: f32, y: f32) {
    canvas.draw_text(
        title,
        Point::new(x, y),
        &Paint::fill(Color::hex("#e2e8f0")),
        16.0,
    );
    
    // Underline
    canvas.draw_rect(
        Rect::new(x, y + 20.0, 100.0, 2.0),
        &Paint::fill(Color::hex("#6366f1")),
    );
}

fn draw_button(canvas: &mut SoftwareBackend, label: &str, x: f32, y: f32, color: Color) {
    let width = 100.0;
    let height = 36.0;
    
    canvas.draw_rounded_rect(
        Rect::new(x, y, width, height),
        BorderRadius::all(8.0),
        &Paint::fill(color),
    );
    
    let text_x = x + (width - label.len() as f32 * 7.0) / 2.0;
    canvas.draw_text(
        label,
        Point::new(text_x, y + 10.0),
        &Paint::fill(Color::WHITE),
        14.0,
    );
}
