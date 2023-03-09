//! VenomUI Demo - Interactive window demo
//!
//! Run with: cargo run --example demo

use venom_app::{App, AppConfig, RenderContext};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point, BorderRadius};

fn main() {
    println!("🚀 Starting VenomUI Demo...");
    
    App::run(
        AppConfig::new("VenomUI Demo")
            .size(800, 600)
            .background(Color::hex("#0f0f1a")),
        render,
    );
}

fn render(canvas: &mut SoftwareBackend, ctx: &RenderContext) {
    let center_x = ctx.width as f32 / 2.0;
    let center_y = ctx.height as f32 / 2.0;
    let time = ctx.frame as f32 * 0.02;

    // Animated background circle
    let radius = 80.0 + (time.sin() * 30.0);
    canvas.draw_circle(
        Point::new(center_x, center_y),
        radius,
        &Paint::fill(Color::hex("#6366f1")),
    );

    // Outer ring
    canvas.draw_circle(
        Point::new(center_x, center_y),
        radius + 20.0,
        &Paint::stroke(Color::hex("#818cf8"), 3.0),
    );

    // Corner cards
    let card_width = 150.0;
    let card_height = 80.0;
    let margin = 30.0;

    // Top-left card
    canvas.draw_rounded_rect(
        Rect::new(margin, margin, card_width, card_height),
        BorderRadius::all(12.0),
        &Paint::fill(Color::hex("#22c55e")),
    );

    // Top-right card
    canvas.draw_rounded_rect(
        Rect::new(ctx.width as f32 - card_width - margin, margin, card_width, card_height),
        BorderRadius::all(12.0),
        &Paint::fill(Color::hex("#ef4444")),
    );

    // Bottom-left card
    canvas.draw_rounded_rect(
        Rect::new(margin, ctx.height as f32 - card_height - margin, card_width, card_height),
        BorderRadius::all(12.0),
        &Paint::fill(Color::hex("#f97316")),
    );

    // Bottom-right card
    canvas.draw_rounded_rect(
        Rect::new(
            ctx.width as f32 - card_width - margin,
            ctx.height as f32 - card_height - margin,
            card_width,
            card_height,
        ),
        BorderRadius::all(12.0),
        &Paint::fill(Color::hex("#0ea5e9")),
    );

    // Mouse cursor indicator
    if let Some(mouse) = ctx.mouse_position {
        canvas.draw_circle(
            mouse,
            12.0,
            &Paint::fill(Color::WHITE),
        );
        canvas.draw_circle(
            mouse,
            12.0,
            &Paint::stroke(Color::hex("#6366f1"), 2.0),
        );
    }

    // Frame counter (placeholder)
    let fps_x = ctx.width as f32 - 100.0;
    canvas.draw_rect(
        Rect::new(fps_x, 10.0, 80.0, 25.0),
        &Paint::fill(Color::rgba(0, 0, 0, 150)),
    );
}
