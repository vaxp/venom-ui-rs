# Layout Guide

VenomUI-RS uses a constraint-based layout system inspired by Flutter.

## Flex Layout

### Row (Horizontal)

```rust
use venom_widgets::{Row, MainAxisAlignment, CrossAxisAlignment};

let row = Row::new()
    .spacing(8.0)
    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .child(widget1)
    .child(widget2)
    .child(widget3);
```

### Column (Vertical)

```rust
use venom_widgets::Column;

let column = Column::new()
    .spacing(16.0)
    .main_axis_alignment(MainAxisAlignment::Start)
    .cross_axis_alignment(CrossAxisAlignment::Stretch)
    .child(header)
    .child(content)
    .child(footer);
```

### Flex (Base)

```rust
use venom_widgets::{Flex, Axis};

// Horizontal flex
let hflex = Flex::row()
    .spacing(8.0)
    .child(item1)
    .child(item2);

// Vertical flex
let vflex = Flex::column()
    .spacing(8.0)
    .child(item1)
    .child(item2);

// With explicit axis
let flex = Flex::new(Axis::Horizontal)
    .spacing(12.0)
    .children(vec![item1, item2, item3]);
```

---

## Main Axis Alignment

Controls distribution along the main axis (horizontal for Row, vertical for Column).

```rust
use venom_widgets::MainAxisAlignment;

// Options:
MainAxisAlignment::Start       // Pack at start
MainAxisAlignment::End         // Pack at end
MainAxisAlignment::Center      // Center children
MainAxisAlignment::SpaceBetween // Space between children
MainAxisAlignment::SpaceAround  // Space around children
MainAxisAlignment::SpaceEvenly  // Equal space everywhere
```

**Visual:**
```
Start:        [A][B][C]............
End:          ............[A][B][C]
Center:       ......[A][B][C]......
SpaceBetween: [A]......[B]......[C]
SpaceAround:  ..[A]....[B]....[C]..
SpaceEvenly:  ...[A]...[B]...[C]...
```

---

## Cross Axis Alignment

Controls alignment perpendicular to main axis.

```rust
use venom_widgets::CrossAxisAlignment;

CrossAxisAlignment::Start    // Align to start edge
CrossAxisAlignment::End      // Align to end edge
CrossAxisAlignment::Center   // Center along cross axis
CrossAxisAlignment::Stretch  // Stretch to fill cross axis
```

---

## Stack

Overlays children on top of each other.

```rust
use venom_widgets::Stack;

let stack = Stack::new()
    .child(background_image)  // Bottom layer
    .child(overlay_gradient)  // Middle layer
    .child(text_content);     // Top layer

// Children are drawn in order (first = bottom, last = top)
```

---

## Spacing

Add uniform spacing between children.

```rust
let row = Row::new()
    .spacing(16.0)  // 16px between each child
    .child(a)
    .child(b)
    .child(c);

// Results in: [A] 16px [B] 16px [C]
```

---

## BoxConstraints

The layout system uses constraints to determine widget sizes.

```rust
use venom_core::BoxConstraints;

// Tight constraints (exact size)
let tight = BoxConstraints::tight(Size::new(100.0, 50.0));

// Loose constraints (max bounds)
let loose = BoxConstraints::loose(Size::new(200.0, 100.0));

// Custom constraints
let custom = BoxConstraints::new()
    .with_min_width(50.0)
    .with_max_width(200.0)
    .with_min_height(30.0)
    .with_max_height(100.0);
```

---

## Alignment

Position children within a container.

```rust
use venom_core::Alignment;

// Predefined alignments
Alignment::TOP_LEFT      // (-1, -1)
Alignment::TOP_CENTER    // (0, -1)
Alignment::TOP_RIGHT     // (1, -1)
Alignment::CENTER_LEFT   // (-1, 0)
Alignment::CENTER        // (0, 0)
Alignment::CENTER_RIGHT  // (1, 0)
Alignment::BOTTOM_LEFT   // (-1, 1)
Alignment::BOTTOM_CENTER // (0, 1)
Alignment::BOTTOM_RIGHT  // (1, 1)

// Custom alignment
let custom = Alignment::new(0.5, -0.5); // Right-ish, top-ish
```

---

## Example: Card Layout

```rust
Container::new()
    .color(Color::hex("#1f2937"))
    .padding_all(16.0)
    .border_radius(12.0)
    .child(
        Column::new()
            .spacing(12.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .child(
                Text::new("Card Title")
                    .size(20.0)
                    .bold()
                    .color(Color::WHITE)
            )
            .child(
                Text::new("This is the card description.")
                    .size(14.0)
                    .color(Color::hex("#9ca3af"))
            )
            .child(
                Row::new()
                    .spacing(8.0)
                    .child(Button::new("Cancel").style(ButtonStyle::Outlined))
                    .child(Button::new("Confirm"))
            )
    )
```

---

## Next: [Styling Guide](./styling.md)
