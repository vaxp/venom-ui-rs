# Widgets Guide

VenomUI-RS provides a collection of widgets for building UIs.

## Basic Widgets

### Container

A versatile box with color, padding, margin, and border radius.

```rust
use venom_widgets::{Container, Color};

let container = Container::new()
    .width(200.0)
    .height(100.0)
    .color(Color::hex("#1a1a2e"))
    .padding_all(16.0)
    .margin_all(8.0)
    .border_radius(12.0)
    .border(Color::WHITE, 2.0)
    .child(my_content);
```

**Builder Methods:**
- `.width(f32)` / `.height(f32)` - Fixed dimensions
- `.color(Color)` - Background color
- `.padding(Insets)` / `.padding_all(f32)` - Inner spacing
- `.margin(Insets)` / `.margin_all(f32)` - Outer spacing
- `.border_radius(f32)` - Rounded corners
- `.border(Color, f32)` - Border color and width
- `.alignment(Alignment)` - Child alignment
- `.child(Widget)` - Add child widget

---

### SizedBox

Forces a specific size.

```rust
use venom_widgets::SizedBox;

// Fixed size
let box1 = SizedBox::new(100.0, 50.0);

// Square
let box2 = SizedBox::square(64.0);

// Spacers
let horizontal_space = SizedBox::width(16.0);
let vertical_space = SizedBox::height(16.0);

// With child
let constrained = SizedBox::new(100.0, 100.0)
    .child(my_widget);
```

---

### Padding

Adds padding around a child.

```rust
use venom_widgets::Padding;

// Uniform padding
let p1 = Padding::all(16.0).child(content);

// Symmetric (horizontal, vertical)
let p2 = Padding::symmetric(24.0, 12.0).child(content);

// Individual sides (top, right, bottom, left)
let p3 = Padding::only(8.0, 16.0, 8.0, 16.0).child(content);
```

---

### Center

Centers its child widget.

```rust
use venom_widgets::Center;

let centered = Center::new()
    .child(my_content);

// With size factors
let partial = Center::new()
    .width_factor(0.8)  // 80% of available width
    .height_factor(0.5) // 50% of available height
    .child(content);
```

---

## Text Widget

Displays styled text.

```rust
use venom_widgets::{Text, TextStyle, FontWeight, TextAlign};

// Simple text
let title = Text::new("Hello, World!");

// Styled text
let styled = Text::new("Important")
    .size(24.0)
    .bold()
    .color(Color::RED)
    .centered();

// With TextStyle
let custom = Text::styled("Custom", TextStyle::new()
    .size(18.0)
    .weight(FontWeight::SemiBold)
    .color(Color::hex("#6366f1"))
    .letter_spacing(1.5)
);
```

**Builder Methods:**
- `.size(f32)` - Font size
- `.color(Color)` - Text color
- `.bold()` / `.italic()` - Font style shortcuts
- `.weight(FontWeight)` - Font weight (Thin to Black)
- `.align(TextAlign)` - Left, Center, Right
- `.centered()` - Center alignment shortcut
- `.max_lines(usize)` - Limit lines
- `.letter_spacing(f32)` - Character spacing

---

## Button Widget

Interactive clickable widget.

```rust
use venom_widgets::{Button, ButtonStyle, ButtonColors};

// Simple button
let btn = Button::new("Click Me");

// With callback
let action_btn = Button::new("Save")
    .on_press(|| {
        println!("Saved!");
    });

// Styled button
let styled = Button::new("Delete")
    .style(ButtonStyle::Outlined)
    .colors(ButtonColors::danger())
    .border_radius(8.0)
    .padding_all(16.0);

// Icon button
let icon_btn = Button::icon(my_icon_widget);
```

**Button Styles:**
- `ButtonStyle::Filled` - Solid background (default)
- `ButtonStyle::Outlined` - Border only
- `ButtonStyle::Text` - Text only
- `ButtonStyle::Elevated` - With shadow

**Color Schemes:**
- `ButtonColors::primary()` - Blue
- `ButtonColors::secondary()` - Gray
- `ButtonColors::danger()` - Red
- `ButtonColors::success()` - Green

---

## Gesture Detection

Handle user input.

### GestureDetector

```rust
use venom_widgets::GestureDetector;

let interactive = GestureDetector::new()
    .on_tap(|| println!("Tapped!"))
    .on_tap_down(|details| println!("Down at {:?}", details.local_position))
    .on_double_tap(|| println!("Double tap!"))
    .on_long_press(|| println!("Long press!"))
    .on_pan_update(|details| println!("Dragging: {:?}", details.delta))
    .on_hover(|pos| println!("Hover at {:?}", pos))
    .child(my_widget);
```

### InkWell

Material-style ripple effect.

```rust
use venom_widgets::InkWell;

let ripple = InkWell::new()
    .on_tap(|| println!("Pressed!"))
    .splash_color(Color::rgba(255, 255, 255, 50))
    .border_radius(8.0)
    .child(content);
```

---

## Image & Icon

### Image

```rust
use venom_widgets::{Image, BoxFit};

// From asset
let logo = Image::asset("images/logo.png")
    .width(200.0)
    .height(100.0)
    .fit(BoxFit::Cover)
    .border_radius(8.0);

// From network
let avatar = Image::network("https://example.com/avatar.jpg")
    .width(50.0)
    .height(50.0)
    .placeholder(Color::GRAY);
```

**BoxFit Options:**
- `BoxFit::Contain` - Scale to fit (default)
- `BoxFit::Cover` - Scale to cover (may crop)
- `BoxFit::Fill` - Stretch to fill
- `BoxFit::ScaleDown` - Only scale down

### Icon

```rust
use venom_widgets::Icon;

let icon = Icon::new("home")
    .size(24.0)
    .color(Color::WHITE);
```

---

## Next: [Layout Guide](./layout.md)
