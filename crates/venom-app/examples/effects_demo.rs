use venom_app::{App, AppConfig, RenderContext};
use venom_core::{
    Color, Alignment, BoxConstraints, Rect, Size, Point,
    Insets, Offset, BorderRadius,
};
use venom_render::{Paint, SoftwareBackend, PaintCanvas};
use venom_widgets::{
    Widget, Container, Row, Column, Stack, Text, SizedBox,
    Card, Chip, Badge, Tooltip, Opacity, Transform, ClipRRect, ClipOval,
    TooltipPosition,
};

struct DemoState {
    // Interactive state
    chip1_selected: bool,
    chip2_deleted: bool,
    hovering_tooltip: bool,
    
    // Input state
    was_pressed: bool,
    
    // Animation
    frame: u64,
}

impl DemoState {
    fn new() -> Self {
        Self {
            chip1_selected: true,
            chip2_deleted: false,
            hovering_tooltip: false,
            was_pressed: false,
            frame: 0,
        }
    }

    fn render(&mut self, canvas: &mut SoftwareBackend, ctx: &RenderContext) {
        // Update frame count
        self.frame = ctx.frame;

        // Input processing
        let pressed = ctx.mouse_pressed;
        let just_pressed = pressed && !self.was_pressed;
        self.was_pressed = pressed;
        
        let mouse_pos = ctx.mouse_position.unwrap_or(Point::new(-1.0, -1.0));

        // HIT REGIONS (Hardcoded matching the layout below)
        // 1. Chip "Select Me"
        //    - Section 1 starts at Y=20 (padding) + 40 (title) = 60
        //    - Card starts roughly at Y=60. Padding=16.
        //    - Text height ~20. Spacing 10.
        //    - Chip Row starts at ~ Y=60+16+20+10 = 106.
        //    - Chip 1 X ~ 20 (screen pad) + 1 (card bump) + 16 (card pad) = ~37.
        //    Let's refine layout to be MORE predictable.
        
        // --- Rendering Logic ---
        // Clear background
        canvas.draw_rect(
            Rect::new(0.0, 0.0, ctx.width as f32, ctx.height as f32),
            &Paint::fill(Color::hex("#121212")),
        );

        let t = (self.frame as f32 * 0.05).sin() * 0.5 + 0.5; // Animation factor

        // We use a Column with loose constraints so it flows naturally
        let content = Container::new()
            .padding_all(20.0)
            .child(
                Column::new()
                    .spacing(30.0) // Space between sections
                    
                    // --- Section 1: Interactive Chips ---
                    .child(
                        Column::new()
                            .spacing(10.0)
                            .child(Self::build_title("Interactive Chips (Click them!)"))
                            .child(
                                Row::new()
                                    .spacing(20.0)
                                    .child(
                                        Card::new()
                                            .color(Color::hex("#1e1e1e"))
                                            .width(400.0)
                                            .child(
                                                Column::new()
                                                    .spacing(15.0)
                                                    .child(Text::new("Filter Options").color(Color::WHITE).size(16.0))
                                                    .child(
                                                        Row::new()
                                                            .spacing(10.0)
                                                            .child(
                                                                Chip::new(if self.chip1_selected { "Selected (Click)" } else { "Select Me" })
                                                                    .selected(self.chip1_selected)
                                                                    .text_color(if self.chip1_selected { Color::WHITE } else { Color::BLACK })
                                                            )
                                                            .child_boxed(
                                                                if !self.chip2_deleted {
                                                                    Box::new(Chip::new("Delete Me")
                                                                        .deletable()
                                                                        .color(Color::hex("#ffcdd2"))
                                                                        .text_color(Color::hex("#c62828")))
                                                                } else {
                                                                    Box::new(
                                                                        Container::new()
                                                                            .child(Text::new("Deleted!").color(Color::hex("#555555")).size(12.0))
                                                                    )
                                                                }
                                                            )
                                                    )
                                            )
                                    )
                            )
                    )

                    // --- Section 2: Badges & Hover Tooltip ---
                    .child(
                        Column::new()
                            .spacing(10.0)
                            .child(Self::build_title("Badges & Tooltips"))
                            .child(
                                Row::new()
                                    .spacing(40.0)
                                    .child(
                                        // Badge
                                        Stack::new()
                                            .child(
                                                Container::new()
                                                    .size(48.0, 48.0)
                                                    .color(Color::hex("#333333"))
                                                    .border_radius(8.0)
                                                    .center()
                                                    .child(Text::new("🔔").size(24.0))
                                            )
                                            .child(
                                                Container::new()
                                                    .margin(Insets { top: -5.0, right: -5.0, ..Default::default() })
                                                    .alignment(Alignment::TOP_RIGHT)
                                                    .child(Badge::new(5))
                                            )
                                    )
                                    .child(
                                        // Dynamic Tooltip Target
                                        // We wrap it in a Container we can vaguely hit-test
                                        // The layout position will heavily depend on previous items:
                                        // Y ~ 20 (root) + 50 (S1 Title) + 120 (S1 Content) + 30 (Gap) + 30 (S2 Title) = 250ish
                                        Tooltip::new(
                                            Container::new()
                                                .color(Color::hex(if self.hovering_tooltip { "#1565c0" } else { "#1976d2" }))
                                                .padding_all(12.0)
                                                .border_radius(6.0)
                                                .child(Text::new("Hover Me ->").color(Color::WHITE)),
                                            "Hello! This is a tooltip."
                                        )
                                        .position(TooltipPosition::Right)
                                        .visible(self.hovering_tooltip)
                                    )
                            )
                    )

                    // --- Section 3: Animations ---
                    .child(
                        Column::new()
                            .spacing(10.0)
                            .child(Self::build_title("Visual Effects"))
                            .child(
                                Row::new()
                                    .spacing(30.0)
                                    .child(
                                        Opacity::new(
                                            Container::new().size(50.0, 50.0).color(Color::hex("#ff9800")),
                                            0.3 + (t * 0.7)
                                        )
                                    )
                                    .child(
                                        Transform::new(
                                            Container::new().size(50.0, 50.0).color(Color::hex("#9c27b0"))
                                        )
                                        .rotate(self.frame as f32 * 3.0)
                                        .scale(0.8 + (t * 0.4))
                                    )
                                    .child(
                                        ClipOval::new(
                                            Container::new().size(50.0, 50.0).color(Color::hex("#009688"))
                                        )
                                    )
                            )
                    )
            );

        // Layout with loose constraints to allow natural sizing
        // (If strictly 800x600, Flex might force stretch)
        let constraints = BoxConstraints::loose(Size::new(ctx.width as f32, ctx.height as f32));
        content.layout(constraints);
        content.paint(canvas, Offset::ZERO);

        // HIT TESTING LOGIC (Manual Rectangle Checks)
        // -------------------------------------------
        // Note: In a real app, you'd use the widget system to handle events.
        // Here we approximate based on the known layout structure defined above.
        
        // Chip 1 ("Select Me")
        // Approx: X[55-150], Y[110-145]
        let chip1_rect = Rect::new(55.0, 110.0, 100.0, 35.0);
        if just_pressed && chip1_rect.contains(mouse_pos) {
            self.chip1_selected = !self.chip1_selected;
        }

        // Chip 2 ("Delete Me")
        // Approx: X[170-270], Y[110-145]
        let chip2_rect = Rect::new(170.0, 110.0, 100.0, 35.0);
        if !self.chip2_deleted && just_pressed && chip2_rect.contains(mouse_pos) {
            self.chip2_deleted = true;
        }

        // Tooltip Hover Target
        // Approx: X[110-210], Y[240-290] (Assuming it's the 2nd item in Badges Row)
        // Corrected logic: Badges Row is after Chips section.
        // Y calc: 20(pad) + 25(text) + 10(gap) + 100(card) + 30(gap) + 25(text) + 10(gap) = ~220
        // X calc: 20(pad) + 48(badge) + 40(gap) = ~108 start
        let tooltip_hit_rect = Rect::new(108.0, 220.0, 120.0, 50.0);
        
        self.hovering_tooltip = tooltip_hit_rect.contains(mouse_pos);

        // DEBUG VISUALIZATION (Optional: draw hit rects to verify positions)
        // Uncomment to debug interaction areas
        /*
        canvas.draw_rect(chip1_rect, &Paint::stroke(Color::hex("#00FF00"), 1.0));
        canvas.draw_rect(chip2_rect, &Paint::stroke(Color::hex("#FF0000"), 1.0));
        canvas.draw_rect(tooltip_hit_rect, &Paint::stroke(Color::hex("#0000FF"), 1.0));
        */

        // Continuous redraw
        ctx.request_redraw(None);
    }
    
    fn build_title(text: &str) -> Text {
        Text::new(text)
            .size(18.0)
            .color(Color::hex("#2196f3")) // Blue accent
    }
}

fn main() {
    let mut state = DemoState::new();
    
    App::run(
        AppConfig::new("VenomUI Effects Demo")
            .size(800, 600)
            .background(Color::hex("#121212")),
        move |canvas, ctx| {
            state.render(canvas, ctx);
        }
    );
}
