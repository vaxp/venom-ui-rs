# Styling Guide

Colors, paint styles, and visual properties.

## Colors

### Creating Colors

```rust
use venom_core::Color;

// From hex string
let purple = Color::hex("#6366f1");
let dark = Color::hex("#1a1a2e");

// From RGB values
let red = Color::rgb(255, 0, 0);

// From RGBA (with alpha)
let transparent_black = Color::rgba(0, 0, 0, 128);

// Predefined colors
let white = Color::WHITE;
let black = Color::BLACK;
let red = Color::RED;
let green = Color::GREEN;
let blue = Color::BLUE;
let transparent = Color::TRANSPARENT;
```

### Color Methods

```rust
let color = Color::hex("#6366f1");

// Modify alpha
let faded = color.with_alpha(128);  // 50% opacity

// Access components
let r = color.r;  // 0-255
let g = color.g;
let b = color.b;
let a = color.a;
```

---

## Paint

Paint defines how shapes are drawn.

### Fill vs Stroke

```rust
use venom_render::Paint;

// Solid fill
let fill = Paint::fill(Color::RED);

// Stroke (outline)
let stroke = Paint::stroke(Color::BLUE, 2.0);
```

### Paint Builder

```rust
let paint = Paint::new()
    .color(Color::hex("#6366f1"))
    .style(PaintStyle::Stroke)
    .stroke_width(3.0)
    .stroke_cap(StrokeCap::Round)
    .stroke_join(StrokeJoin::Round)
    .anti_alias(true);
```

### Stroke Caps

```rust
use venom_render::StrokeCap;

StrokeCap::Butt   // Flat end at endpoint
StrokeCap::Round  // Rounded end, extends past
StrokeCap::Square // Square end, extends past
```

### Stroke Joins

```rust
use venom_render::StrokeJoin;

StrokeJoin::Miter // Sharp corners
StrokeJoin::Round // Rounded corners
StrokeJoin::Bevel // Flattened corners
```

---

## Insets (Padding/Margin)

```rust
use venom_core::Insets;

// Uniform on all sides
let uniform = Insets::all(16.0);

// Symmetric (horizontal, vertical)
let symmetric = Insets::symmetric(24.0, 12.0);

// Individual sides (top, right, bottom, left)
let custom = Insets::new(8.0, 16.0, 8.0, 16.0);

// Zero insets
let none = Insets::ZERO;

// Access values
let total_horizontal = insets.horizontal_total();
let total_vertical = insets.vertical_total();
```

---

## Border Radius

```rust
use venom_core::BorderRadius;

// Same radius on all corners
let uniform = BorderRadius::all(12.0);

// Different radius per corner
let custom = BorderRadius::new(8.0, 16.0, 8.0, 16.0);
// Order: top_left, top_right, bottom_right, bottom_left

// Only certain corners
let top_only = BorderRadius::new(12.0, 12.0, 0.0, 0.0);
```

---

## Text Styling

```rust
use venom_widgets::{TextStyle, FontWeight, FontStyle};

let style = TextStyle::new()
    .size(18.0)
    .color(Color::WHITE)
    .weight(FontWeight::Bold)
    .style(FontStyle::Italic)
    .letter_spacing(1.5)
    .line_height(1.4);

// Use with Text
let text = Text::styled("Styled text", style);
```

### Font Weights

```rust
FontWeight::Thin      // 100
FontWeight::Light     // 300
FontWeight::Normal    // 400 (default)
FontWeight::Medium    // 500
FontWeight::SemiBold  // 600
FontWeight::Bold      // 700
FontWeight::ExtraBold // 800
FontWeight::Black     // 900
```

---

## Button Colors

```rust
use venom_widgets::ButtonColors;

// Predefined schemes
let primary = ButtonColors::primary();   // Purple/indigo
let secondary = ButtonColors::secondary(); // Gray
let danger = ButtonColors::danger();     // Red
let success = ButtonColors::success();   // Green

// Custom colors accessed by state
let bg = colors.background_for_state(ButtonState::Hovered);
let fg = colors.foreground_for_state(ButtonState::Disabled);
```

---

## Example: Themed Card

```rust
// Define a theme
struct Theme {
    background: Color,
    surface: Color,
    primary: Color,
    text: Color,
    text_secondary: Color,
}

let dark_theme = Theme {
    background: Color::hex("#0f0f1a"),
    surface: Color::hex("#1f2937"),
    primary: Color::hex("#6366f1"),
    text: Color::WHITE,
    text_secondary: Color::hex("#9ca3af"),
};

// Use theme
Container::new()
    .color(theme.surface)
    .child(
        Text::new("Hello")
            .color(theme.text)
    )
```

---

## Next: [Rendering Guide](./rendering.md)
