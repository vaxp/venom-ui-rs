# VenomUI-RS Documentation

> [!NOTE]
> **Experimental Prototype in Rust:**  
> A great experiment and solid foundation for exploring UI framework design in Rust. This repository has been officially discontinued after completing all experiments in Rust and was not adopted for the final framework. C++ was chosen instead for the primary framework: **[ENKI](https://github.com/vaxp/enki)**.

Welcome to VenomUI-RS - an experimental declarative UI framework in pure Rust.

## Quick Start

```rust
use venom_app::{App, AppConfig};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect};

fn main() {
    App::run(AppConfig::new("My App").size(800, 600), |canvas, ctx| {
        canvas.draw_rect(
            Rect::new(100.0, 100.0, 200.0, 100.0),
            &Paint::fill(Color::RED)
        );
    });
}
```

## Documentation

- [Widgets Guide](./widgets.md) - All available widgets
- [Layout Guide](./layout.md) - Row, Column, Stack, Flex
- [Styling Guide](./styling.md) - Colors, Paint, Borders
- [Rendering Guide](./rendering.md) - Canvas and drawing
- [App Guide](./app.md) - Window and event handling

## Crates

| Crate | Description |
|-------|-------------|
| `venom-core` | Core types (Color, Size, Point, etc.) |
| `venom-render` | Rendering abstraction + SoftwareBackend |
| `venom-widgets` | Widget system (Container, Text, Button, etc.) |
| `venom-app` | Window management with winit |

## Run Demo

```bash
cargo run --example demo -p venom-app
```
