# App Guide

Window management and event handling with venom-app.

## Quick Start

```rust
use venom_app::{App, AppConfig};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect};

fn main() {
    App::run(
        AppConfig::new("My App").size(800, 600),
        |canvas, ctx| {
            // Your drawing code here
            canvas.draw_rect(
                Rect::new(100.0, 100.0, 200.0, 100.0),
                &Paint::fill(Color::RED)
            );
        }
    );
}
```

---

## AppConfig

Configure window properties.

```rust
use venom_app::AppConfig;

let config = AppConfig::new("Window Title")
    .size(1280, 720)        // Window dimensions
    .resizable(true)        // Allow resizing
    .background(Color::hex("#0f0f1a")); // Default background

// Use defaults
let default_config = AppConfig::default();
```

### Available Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `title` | String | "VenomUI App" | Window title |
| `width` | u32 | 800 | Initial width |
| `height` | u32 | 600 | Initial height |
| `resizable` | bool | true | Allow resizing |
| `vsync` | bool | true | Vertical sync |
| `background` | Color | #1a1a2e | Background color |

---

## RenderContext

The callback receives a context with frame information.

```rust
App::run(config, |canvas, ctx| {
    // Window dimensions
    let width = ctx.width;
    let height = ctx.height;
    let size = ctx.size();

    // Mouse position (if inside window)
    if let Some(mouse) = ctx.mouse_position {
        println!("Mouse at: {:?}", mouse);
    }

    // Frame info
    let frame_number = ctx.frame;
    let delta = ctx.delta_time; // Seconds since last frame
});
```

---

## Mouse Tracking

```rust
App::run(config, |canvas, ctx| {
    // Draw cursor indicator
    if let Some(mouse) = ctx.mouse_position {
        canvas.draw_circle(
            mouse,
            10.0,
            &Paint::fill(Color::WHITE)
        );
    }
});
```

---

## Animation

Use `ctx.frame` for simple animations.

```rust
App::run(config, |canvas, ctx| {
    let time = ctx.frame as f32 * 0.02;
    
    // Pulsing circle
    let radius = 50.0 + (time.sin() * 20.0);
    canvas.draw_circle(
        Point::new(400.0, 300.0),
        radius,
        &Paint::fill(Color::hex("#6366f1"))
    );
    
    // Moving object
    let x = 400.0 + (time * 2.0).cos() * 100.0;
    let y = 300.0 + (time * 2.0).sin() * 100.0;
    canvas.draw_circle(
        Point::new(x, y),
        20.0,
        &Paint::fill(Color::RED)
    );
});
```

---

## Window Events

The app handles these events automatically:

| Event | Behavior |
|-------|----------|
| Close | Exits application |
| Resize | Recreates canvas, redraws |
| Cursor Move | Updates `ctx.mouse_position` |
| Cursor Leave | Sets mouse position to None |
| Redraw | Calls your render function |

---

## Run Demo

```bash
cargo run --example demo -p venom-app
```

Features:
- Animated pulsing circle
- 4 colored corner cards  
- Mouse cursor tracking
- 60fps rendering

---

## Full Example

```rust
use venom_app::{App, AppConfig, RenderContext};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point, BorderRadius};

fn main() {
    App::run(
        AppConfig::new("VenomUI Demo")
            .size(800, 600)
            .background(Color::hex("#0f0f1a")),
        render,
    );
}

fn render(canvas: &mut SoftwareBackend, ctx: &RenderContext) {
    let cx = ctx.width as f32 / 2.0;
    let cy = ctx.height as f32 / 2.0;
    
    // Animated background
    let time = ctx.frame as f32 * 0.02;
    let radius = 80.0 + (time.sin() * 30.0);
    
    canvas.draw_circle(
        Point::new(cx, cy),
        radius,
        &Paint::fill(Color::hex("#6366f1"))
    );
    
    // UI card
    canvas.draw_rounded_rect(
        Rect::new(30.0, 30.0, 150.0, 80.0),
        BorderRadius::all(12.0),
        &Paint::fill(Color::hex("#22c55e"))
    );
    
    // Mouse cursor
    if let Some(mouse) = ctx.mouse_position {
        canvas.draw_circle(mouse, 8.0, &Paint::fill(Color::WHITE));
    }
}
```
