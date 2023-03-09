# Rendering Guide

Drawing shapes and graphics with VenomUI-RS.

## Software Backend

The `SoftwareBackend` uses tiny-skia for CPU rendering.

```rust
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point};

// Create a canvas
let mut backend = SoftwareBackend::new(800, 600);

// Clear with background color
backend.clear(Color::hex("#0f0f1a"));

// Draw shapes
backend.draw_rect(
    Rect::new(10.0, 10.0, 100.0, 50.0),
    &Paint::fill(Color::RED)
);

// Save as PNG
backend.save_png(std::path::Path::new("output.png"))?;
```

---

## Drawing Shapes

### Rectangle

```rust
let rect = Rect::new(x, y, width, height);
canvas.draw_rect(rect, &Paint::fill(Color::BLUE));
```

### Rounded Rectangle

```rust
use venom_core::BorderRadius;

canvas.draw_rounded_rect(
    Rect::new(10.0, 10.0, 100.0, 50.0),
    BorderRadius::all(12.0),
    &Paint::fill(Color::GREEN)
);
```

### Circle

```rust
canvas.draw_circle(
    Point::new(100.0, 100.0), // center
    50.0,                      // radius
    &Paint::fill(Color::RED)
);
```

### Oval

```rust
canvas.draw_oval(
    Rect::new(10.0, 10.0, 100.0, 50.0),
    &Paint::fill(Color::PURPLE)
);
```

### Line

```rust
canvas.draw_line(
    Point::new(0.0, 0.0),
    Point::new(100.0, 100.0),
    &Paint::stroke(Color::WHITE, 2.0)
);
```

---

## Paths

For complex shapes, use paths.

```rust
use venom_render::Path;

let path = Path::new()
    .move_to(50.0, 0.0)
    .line_to(100.0, 100.0)
    .line_to(0.0, 100.0)
    .close();

canvas.draw_path(&path, &Paint::fill(Color::YELLOW));
```

### Path Commands

```rust
path.move_to(x, y)      // Start new subpath
path.line_to(x, y)      // Straight line
path.quad_to(cx, cy, x, y)  // Quadratic bezier
path.cubic_to(c1x, c1y, c2x, c2y, x, y) // Cubic bezier
path.close()            // Close current subpath
```

### Built-in Shapes

```rust
// Rectangle path
let rect_path = Path::rect(Rect::new(0.0, 0.0, 100.0, 50.0));

// Rounded rectangle path
let rounded = Path::rounded_rect(rect, BorderRadius::all(8.0));

// Circle path
let circle = Path::circle(Point::new(50.0, 50.0), 30.0);

// Oval path
let oval = Path::oval(Rect::new(0.0, 0.0, 100.0, 50.0));
```

---

## Transform

Apply transformations to drawings.

```rust
use venom_render::Transform;

// Create transforms
let translate = Transform::translation(100.0, 50.0);
let scale = Transform::scaling(2.0, 2.0);
let rotate = Transform::rotation(45.0); // degrees

// Combine transforms
let combined = Transform::identity()
    .translate(100.0, 100.0)
    .rotate(45.0)
    .scale(1.5, 1.5);

// Apply to canvas
canvas.set_transform(&combined);
canvas.draw_rect(rect, &paint);
```

---

## Save/Restore

Save and restore canvas state.

```rust
// Save current state (transform, clip)
canvas.save();

// Apply changes
canvas.translate(100.0, 100.0);
canvas.draw_rect(rect, &paint);

// Restore previous state
canvas.restore();

// Now transform is back to before save()
```

---

## Export

### PNG Export

```rust
// Get PNG bytes
let png_bytes = backend.encode_png();

// Save to file
backend.save_png(std::path::Path::new("output.png"))?;
```

### Raw Pixels

```rust
// Get raw RGBA pixels (premultiplied alpha)
let pixels: &[u8] = backend.pixels();

// Pixel format: [R, G, B, A, R, G, B, A, ...]
// Total bytes = width * height * 4
```

---

## PaintCanvas Trait

Widgets use the `PaintCanvas` trait for drawing:

```rust
pub trait PaintCanvas {
    fn clear(&mut self, color: Color);
    fn draw_rect(&mut self, rect: Rect, paint: &Paint);
    fn draw_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, paint: &Paint);
    fn draw_circle(&mut self, center: Point, radius: f32, paint: &Paint);
    fn draw_oval(&mut self, rect: Rect, paint: &Paint);
    fn draw_line(&mut self, p1: Point, p2: Point, paint: &Paint);
    fn draw_path(&mut self, path: &Path, paint: &Paint);
    fn draw_text(&mut self, text: &str, position: Point, paint: &Paint, size: f32);
    fn set_transform(&mut self, transform: &Transform);
    fn save(&mut self);
    fn restore(&mut self);
    fn translate(&mut self, dx: f32, dy: f32);
}
```

---

## Next: [App Guide](./app.md)
