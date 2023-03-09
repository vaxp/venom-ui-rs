//! New Widgets Demo - Showcases all newly implemented widgets
//!
//! This example demonstrates the new widgets added to venom-widgets.
//! Now interactive!
//!
//! Run with: cargo run --example new_widgets_demo

use venom_app::{App, AppConfig, RenderContext};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};
use venom_core::{Color, Rect, Point, BorderRadius};
use venom_widgets::Cursor; // Added Cursor

// ============================================================================
// APPLICATION STATE
// ============================================================================

struct AppState {
    selected_tab: usize,
    selected_chip: Option<usize>,
    selected_bottom_nav: usize,
    dropdown_open: bool,
    selected_dropdown_idx: Option<usize>,
    
    // Checkbox states (simulated for list)
    list_checks: [bool; 3],
    
    // Animation toggle
    anim_paused: bool,
    
    last_click_frame: u64,
}

impl AppState {
    fn new() -> Self {
        Self {
            selected_tab: 0,
            selected_chip: Some(0), // Rust selected by default
            selected_bottom_nav: 0,
            dropdown_open: false,
            selected_dropdown_idx: Some(0),
            list_checks: [false, true, false],
            anim_paused: false,
            last_click_frame: 0,
        }
    }
}

// Global state
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
    println!("🚀 Starting New Widgets Demo...");
    
    App::run(
        AppConfig::new("VenomUI - New Widgets Demo")
            .size(1200, 900)
            .background(Color::hex("#1a1a2e")),
        render,
    );
}

fn render(canvas: &mut SoftwareBackend, ctx: &RenderContext) {
    let width = ctx.width as f32;
    // let height = ctx.height as f32; // Unused warning fix
    let height = ctx.height as f32;

    let state = get_state();
    
    // Handle input
    let cursor = ctx.mouse_position.map(|p| Cursor::at(p.x, p.y)).unwrap_or(Cursor::unavailable());
    handle_input(state, cursor, ctx.mouse_pressed, ctx.frame, width, height);

    // =========================================================================
    // APP BAR (Top)
    // =========================================================================
    let app_bar_height = 56.0;
    canvas.draw_rect(
        Rect::new(0.0, 0.0, width, app_bar_height),
        &Paint::fill(Color::rgb(63, 81, 181)),  // Indigo
    );
    
    canvas.draw_text(
        "New Widgets Demo - Interactive!",
        Point::new(16.0, app_bar_height / 2.0 - 8.0),
        &Paint::fill(Color::WHITE),
        18.0,
    );
    
    // =========================================================================
    // LEFT PANEL - Layout Helpers Demo
    // =========================================================================
    let panel_y = app_bar_height + 16.0;
    let panel_width = width / 2.0 - 24.0;
    
    // Section Title: Layout Helpers
    canvas.draw_text(
        "📐 Layout Helpers",
        Point::new(16.0, panel_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    // Spacer Demo - Three boxes with space between
    let spacer_y = panel_y + 30.0;
    canvas.draw_rounded_rect(
        Rect::new(16.0, spacer_y, 60.0, 40.0),
        BorderRadius::all(8.0),
        &Paint::fill(Color::rgb(255, 87, 34)),  // Deep Orange
    );
    canvas.draw_text("Spacer →", Point::new(90.0, spacer_y + 15.0), &Paint::fill(Color::rgba(255, 255, 255, 180)), 12.0);
    canvas.draw_rounded_rect(
        Rect::new(200.0, spacer_y, 60.0, 40.0),
        BorderRadius::all(8.0),
        &Paint::fill(Color::rgb(76, 175, 80)),  // Green
    );
    
    // Divider
    let divider_y = spacer_y + 60.0;
    canvas.draw_rect(
        Rect::new(16.0, divider_y, panel_width, 1.0),
        &Paint::fill(Color::rgba(255, 255, 255, 50)),
    );
    
    // Aspect Ratio Demo
    let aspect_y = divider_y + 20.0;
    canvas.draw_text(
        "📐 AspectRatio (16:9, 1:1)",
        Point::new(16.0, aspect_y),
        &Paint::fill(Color::WHITE),
        14.0,
    );
    // 16:9 box
    canvas.draw_rounded_rect(
        Rect::new(16.0, aspect_y + 25.0, 160.0, 90.0),
        BorderRadius::all(8.0),
        &Paint::fill(Color::rgb(0, 188, 212)),
    );
    canvas.draw_text("16:9", Point::new(70.0, aspect_y + 60.0), &Paint::fill(Color::WHITE), 14.0);
    // 1:1 box
    canvas.draw_rounded_rect(
        Rect::new(190.0, aspect_y + 25.0, 80.0, 80.0),
        BorderRadius::all(8.0),
        &Paint::fill(Color::rgb(156, 39, 176)),
    );
    canvas.draw_text("1:1", Point::new(215.0, aspect_y + 60.0), &Paint::fill(Color::WHITE), 14.0);
    
    // =========================================================================
    // CARDS SECTION
    // =========================================================================
    let cards_y = aspect_y + 140.0;
    canvas.draw_text(
        "🎴 Card Widgets",
        Point::new(16.0, cards_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    // Elevated Card
    let card_y = cards_y + 30.0;
    // Shadow
    canvas.draw_rounded_rect(
        Rect::new(18.0, card_y + 4.0, 150.0, 100.0),
        BorderRadius::all(12.0),
        &Paint::fill(Color::rgba(0, 0, 0, 50)),
    );
    // Card bg
    canvas.draw_rounded_rect(
        Rect::new(16.0, card_y, 150.0, 100.0),
        BorderRadius::all(12.0),
        &Paint::fill(Color::rgb(45, 45, 70)),
    );
    canvas.draw_text("Elevated Card", Point::new(30.0, card_y + 30.0), &Paint::fill(Color::WHITE), 14.0);
    canvas.draw_text("With shadow", Point::new(30.0, card_y + 55.0), &Paint::fill(Color::rgba(255, 255, 255, 150)), 12.0);
    
    // Outlined Card
    canvas.draw_rounded_rect(
        Rect::new(180.0, card_y, 150.0, 100.0),
        BorderRadius::all(12.0),
        &Paint::stroke(Color::rgba(255, 255, 255, 100), 1.0),
    );
    canvas.draw_text("Outlined Card", Point::new(194.0, card_y + 30.0), &Paint::fill(Color::WHITE), 14.0);
    canvas.draw_text("No shadow", Point::new(194.0, card_y + 55.0), &Paint::fill(Color::rgba(255, 255, 255, 150)), 12.0);
    
    // =========================================================================
    // CHIPS & BADGES
    // =========================================================================
    let chips_y = card_y + 120.0;
    canvas.draw_text(
        "🏷️ Chips & Badges (Clickable)",
        Point::new(16.0, chips_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    // Chips
    let chip_y = chips_y + 30.0;
    // Chip 1
    let chip1_bg = if state.selected_chip == Some(0) { Color::rgb(255, 152, 0) } else { Color::rgba(255, 152, 0, 100) };
    canvas.draw_rounded_rect(
        Rect::new(16.0, chip_y, 70.0, 32.0),
        BorderRadius::all(16.0),
        &Paint::fill(chip1_bg),
    );
    canvas.draw_text("Rust", Point::new(32.0, chip_y + 10.0), &Paint::fill(Color::WHITE), 13.0);
    
    // Chip 2 (selected)
    let chip2_bg = if state.selected_chip == Some(1) { Color::rgb(33, 150, 243) } else { Color::rgba(33, 150, 243, 100) };
    canvas.draw_rounded_rect(
        Rect::new(95.0, chip_y, 80.0, 32.0),
        BorderRadius::all(16.0),
        &Paint::fill(chip2_bg),
    );
    canvas.draw_text("VenomUI", Point::new(106.0, chip_y + 10.0), &Paint::fill(Color::WHITE), 13.0);
    
    // Badge (number)
    canvas.draw_rounded_rect(
        Rect::new(185.0, chip_y + 5.0, 28.0, 22.0),
        BorderRadius::all(11.0),
        &Paint::fill(Color::rgb(255, 59, 48)),
    );
    canvas.draw_text("42", Point::new(191.0, chip_y + 11.0), &Paint::fill(Color::WHITE), 12.0);
    
    // Badge (dot)
    canvas.draw_circle(
        Point::new(230.0, chip_y + 16.0),
        6.0,
        &Paint::fill(Color::rgb(76, 175, 80)),
    );
    
    // =========================================================================
    // RIGHT PANEL - Navigation & Lists
    // =========================================================================
    let right_x = width / 2.0 + 8.0;
    
    // Section Title: Tab Bar
    canvas.draw_text(
        "📑 TabBar (Click to switch)",
        Point::new(right_x, panel_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    // Tab Bar
    let tab_y = panel_y + 30.0;
    canvas.draw_rect(
        Rect::new(right_x, tab_y, 350.0, 48.0),
        &Paint::fill(Color::rgb(63, 81, 181)),
    );
    // Tabs
    let tab_names = ["Home", "Search", "Settings"];
    for (i, name) in tab_names.iter().enumerate() {
        let tab_x = right_x + i as f32 * 116.0;
        let selected = i == state.selected_tab;
        let alpha = if selected { 255 } else { 178 };
        
        canvas.draw_text(
            name,
            Point::new(tab_x + 40.0, tab_y + 18.0),
            &Paint::fill(Color::rgba(255, 255, 255, alpha)),
            14.0,
        );
        
        // Selected indicator
        if selected {
            canvas.draw_rect(
                Rect::new(tab_x, tab_y + 45.0, 116.0, 3.0),
                &Paint::fill(Color::WHITE),
            );
        }
    }
    
    // =========================================================================
    // LIST VIEW
    // =========================================================================
    let list_y = tab_y + 70.0;
    canvas.draw_text(
        "📋 ListView (Click items)",
        Point::new(right_x, list_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    // List items
    let items = [
        ("List Item 1", "With subtitle"),
        ("List Item 2", "Selectable item"),
        ("List Item 3", "Another item"),
    ];
    
    for (i, (title, subtitle)) in items.iter().enumerate() {
        let item_y = list_y + 30.0 + i as f32 * 56.0;
        let selected = state.list_checks[i];
        
        if selected {
            canvas.draw_rect(
                Rect::new(right_x, item_y, 350.0, 54.0),
                &Paint::fill(Color::rgba(33, 150, 243, 30)),
            );
        }
        
        canvas.draw_text(title, Point::new(right_x + 16.0, item_y + 12.0), &Paint::fill(Color::WHITE), 15.0);
        canvas.draw_text(subtitle, Point::new(right_x + 16.0, item_y + 32.0), &Paint::fill(Color::rgba(255, 255, 255, 150)), 13.0);
        
        // Checkmark indicator if selected
        if selected {
            canvas.draw_text("✓", Point::new(right_x + 320.0, item_y + 20.0), &Paint::fill(Color::rgb(76, 175, 80)), 16.0);
        }

        // Divider
        canvas.draw_rect(
            Rect::new(right_x + 16.0, item_y + 53.0, 334.0, 1.0),
            &Paint::fill(Color::rgba(255, 255, 255, 30)),
        );
    }
    
    // =========================================================================
    // GRID VIEW
    // =========================================================================
    let grid_y = list_y + 210.0;
    canvas.draw_text(
        "🔲 GridView",
        Point::new(right_x, grid_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    let grid_colors = [
        Color::rgb(255, 87, 34),
        Color::rgb(76, 175, 80),
        Color::rgb(33, 150, 243),
        Color::rgb(156, 39, 176),
    ];
    
    for (i, color) in grid_colors.iter().enumerate() {
        let row = i / 2;
        let col = i % 2;
        let cell_x = right_x + col as f32 * 85.0;
        let cell_y = grid_y + 30.0 + row as f32 * 85.0;
        
        canvas.draw_rounded_rect(
            Rect::new(cell_x, cell_y, 80.0, 80.0),
            BorderRadius::all(8.0),
            &Paint::fill(*color),
        );
        canvas.draw_text(&(i + 1).to_string(), Point::new(cell_x + 35.0, cell_y + 35.0), &Paint::fill(Color::WHITE), 18.0);
    }
    
    // =========================================================================
    // PICKERS
    // =========================================================================
    let pickers_y = grid_y + 210.0;
    canvas.draw_text(
        "🔽 Pickers",
        Point::new(right_x, pickers_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    // Dropdown
    canvas.draw_rounded_rect(
        Rect::new(right_x, pickers_y + 30.0, 200.0, 44.0),
        BorderRadius::all(4.0),
        &Paint::fill(Color::rgb(45, 45, 70)),
    );
    canvas.draw_rounded_rect(
        Rect::new(right_x, pickers_y + 30.0, 200.0, 44.0),
        BorderRadius::all(4.0),
        &Paint::stroke(Color::rgba(255, 255, 255, 50), 1.0),
    );
    
    let options = ["Option 1", "Option 2", "Option 3"];
    let selected_text = state.selected_dropdown_idx.map(|i| options[i]).unwrap_or("Select...");
    
    canvas.draw_text(selected_text, Point::new(right_x + 12.0, pickers_y + 45.0), &Paint::fill(Color::WHITE), 14.0);
    canvas.draw_text(if state.dropdown_open { "▲" } else { "▼" }, Point::new(right_x + 175.0, pickers_y + 45.0), &Paint::fill(Color::rgba(255, 255, 255, 150)), 12.0);
    
    // Dropdown Menu (Overlay)
    if state.dropdown_open {
         // Menu BG
         canvas.draw_rounded_rect(
            Rect::new(right_x, pickers_y + 80.0, 200.0, 100.0),
            BorderRadius::all(4.0),
            &Paint::fill(Color::rgb(35, 35, 50)),
        );
        // Menu Shadow
        canvas.draw_rounded_rect(
            Rect::new(right_x, pickers_y + 80.0, 200.0, 100.0),
            BorderRadius::all(4.0),
            &Paint::stroke(Color::rgba(0, 0, 0, 100), 1.0),
        );
        
        for (i, opt) in options.iter().enumerate() {
            let opt_y = pickers_y + 80.0 + i as f32 * 30.0;
            // Highlight selected
            if Some(i) == state.selected_dropdown_idx {
                 canvas.draw_rect(
                    Rect::new(right_x + 2.0, opt_y + 2.0, 196.0, 28.0),
                    &Paint::fill(Color::rgba(63, 81, 181, 100)),
                 );
            }
            canvas.draw_text(opt, Point::new(right_x + 12.0, opt_y + 20.0), &Paint::fill(Color::WHITE), 13.0);
        }
    }

    // Search Field
    canvas.draw_rounded_rect(
        Rect::new(right_x + 210.0, pickers_y + 30.0, 170.0, 44.0),
        BorderRadius::all(22.0),
        &Paint::fill(Color::rgba(255, 255, 255, 20)),
    );
    canvas.draw_text("🔍", Point::new(right_x + 222.0, pickers_y + 43.0), &Paint::fill(Color::rgba(255, 255, 255, 150)), 14.0);
    canvas.draw_text("Search...", Point::new(right_x + 250.0, pickers_y + 45.0), &Paint::fill(Color::rgba(255, 255, 255, 100)), 14.0);
    
    // =========================================================================
    // ANIMATION & EFFECTS
    // =========================================================================
    let anim_y = pickers_y + 100.0;
    canvas.draw_text(
        "✨ Animation & Effects (Click to pause)",
        Point::new(right_x, anim_y),
        &Paint::fill(Color::WHITE),
        16.0,
    );
    
    // Button to pause
    let pause_color = if state.anim_paused { Color::rgb(244, 67, 54) } else { Color::rgb(76, 175, 80) };
    canvas.draw_rounded_rect(
        Rect::new(right_x + 280.0, anim_y, 60.0, 24.0),
        BorderRadius::all(4.0),
        &Paint::fill(pause_color),
    );
    canvas.draw_text(if state.anim_paused { "PAUSED" } else { "RUNNING" }, Point::new(right_x + 290.0, anim_y + 16.0), &Paint::fill(Color::WHITE), 11.0);


    let time = if state.anim_paused { 0.0 } else { ctx.frame as f32 * 0.03 };
    
    // Animated Scale (pulsing)
    let scale = 1.0 + (time.sin() * 0.15);
    let anim_size = 50.0 * scale;
    let anim_offset = (50.0 - anim_size) / 2.0;
    canvas.draw_rounded_rect(
        Rect::new(right_x + anim_offset, anim_y + 30.0 + anim_offset, anim_size, anim_size),
        BorderRadius::all(8.0),
        &Paint::fill(Color::rgb(255, 193, 7)),
    );
    
    // Opacity variation
    let opacity = ((time.sin() + 1.0) * 0.5 * 255.0) as u8;
    canvas.draw_rounded_rect(
        Rect::new(right_x + 70.0, anim_y + 30.0, 50.0, 50.0),
        BorderRadius::all(8.0),
        &Paint::fill(Color::rgba(244, 67, 54, opacity)),
    );
    
    // Rotating (simulated with position)
    let rot_x = right_x + 140.0 + (time * 2.0).cos() * 10.0;
    let rot_y = anim_y + 30.0 + (time * 2.0).sin() * 10.0;
    canvas.draw_rounded_rect(
        Rect::new(rot_x, rot_y, 50.0, 50.0),
        BorderRadius::all(8.0),
        &Paint::fill(Color::rgb(33, 150, 243)),
    );
    
    // ClipRRect (circle clip)
    canvas.draw_circle(
        Point::new(right_x + 235.0, anim_y + 55.0),
        25.0,
        &Paint::fill(Color::rgb(76, 175, 80)),
    );
    
    // =========================================================================
    // BOTTOM NAV BAR
    // =========================================================================
    let nav_height = 56.0;
    let nav_y = height - nav_height;
    
    // Shadow
    canvas.draw_rect(
        Rect::new(0.0, nav_y - 1.0, width, 1.0),
        &Paint::fill(Color::rgba(0, 0, 0, 50)),
    );
    
    // Background
    canvas.draw_rect(
        Rect::new(0.0, nav_y, width, nav_height),
        &Paint::fill(Color::rgb(30, 30, 50)),
    );
    
    // Nav items
    let nav_items = ["🏠 Home", "🔍 Search", "👤 Profile", "⚙️ Settings"];
    let item_width = width / 4.0;
    
    for (i, item) in nav_items.iter().enumerate() {
        let item_x = i as f32 * item_width;
        let selected = i == state.selected_bottom_nav;
        let color = if selected {
            Color::rgb(63, 81, 181)
        } else {
            Color::rgba(255, 255, 255, 128)
        };
        
        // Selection highlight bg
        if selected {
             canvas.draw_rect(
                Rect::new(item_x, nav_y, item_width, 2.0),
                &Paint::fill(Color::rgb(63, 81, 181)),
            );
             canvas.draw_rect(
                Rect::new(item_x, nav_y, item_width, nav_height),
                &Paint::fill(Color::rgba(63, 81, 181, 30)),
            );
        }

        canvas.draw_text(
            item,
            Point::new(item_x + item_width / 2.0 - 30.0, nav_y + 22.0),
            &Paint::fill(color),
            13.0,
        );
    }
    
    // =========================================================================
    // FOOTER INFO
    // =========================================================================
    canvas.draw_text(
        &format!("Frame: {} | Widgets: 31 | Clickable!", ctx.frame),
        Point::new(16.0, height - 70.0),
        &Paint::fill(Color::rgba(255, 255, 255, 100)),
        11.0,
    );
    
    // Mouse Cursor
    if let Some(pos) = ctx.mouse_position {
        canvas.draw_circle(pos, 5.0, &Paint::fill(Color::WHITE));
    }
}

fn handle_input(state: &mut AppState, cursor: Cursor, pressed: bool, frame: u64, width: f32, height: f32) {
    if !pressed { return; }
    
    // Debounce
    if frame - state.last_click_frame < 10 { return; }
    state.last_click_frame = frame;
    
    let app_bar_height = 56.0;
    let panel_y = app_bar_height + 16.0;
    
    // ------------------------------------------------------------------------
    // Chips Logic
    // ------------------------------------------------------------------------
    // Re-calculate layout (simplified for hit testing)
    // In a real app we would cache layout rects or use a proper event system
    let aspect_y = panel_y + 30.0 + 40.0 + 60.0 + 20.0;
    let cards_y = aspect_y + 140.0;
    let chips_y = cards_y + 120.0;
    let chip_y = chips_y + 30.0;
    
    // Chip 1 Hit Test
    if cursor.is_over(16.0, chip_y, 70.0, 32.0) {
        state.selected_chip = Some(0);
        return;
    }
    // Chip 2 Hit Test
    if cursor.is_over(95.0, chip_y, 80.0, 32.0) {
        state.selected_chip = Some(1);
        return;
    }
    
    // ------------------------------------------------------------------------
    // Tabs Logic
    // ------------------------------------------------------------------------
    let right_x = width / 2.0 + 8.0;
    let tab_y = panel_y + 30.0;
    
    for i in 0..3 {
         let tab_x = right_x + i as f32 * 116.0;
         if cursor.is_over(tab_x, tab_y, 116.0, 48.0) {
             state.selected_tab = i;
             return;
         }
    }
    
    // ------------------------------------------------------------------------
    // List Logic
    // ------------------------------------------------------------------------
    let list_y = tab_y + 70.0;
    for i in 0..3 {
        let item_y = list_y + 30.0 + i as f32 * 56.0;
        if cursor.is_over(right_x, item_y, 350.0, 56.0) {
            state.list_checks[i] = !state.list_checks[i];
            return;
        }
    }
    
    // ------------------------------------------------------------------------
    // Picker Logic
    // ------------------------------------------------------------------------
    let grid_y = list_y + 210.0;
    let pickers_y = grid_y + 210.0;
    
    // Dropdown options (if open)
    if state.dropdown_open {
        for i in 0..3 {
            let opt_y = pickers_y + 80.0 + i as f32 * 30.0;
            if cursor.is_over(right_x, opt_y, 200.0, 30.0) {
                state.selected_dropdown_idx = Some(i);
                state.dropdown_open = false;
                return;
            }
        }
    }
    
    // Dropdown toggle
    if cursor.is_over(right_x, pickers_y + 30.0, 200.0, 44.0) {
        state.dropdown_open = !state.dropdown_open;
        return;
    }
    
    // Close dropdown if clicked elsewhere
    if state.dropdown_open {
        state.dropdown_open = false;
    }
    
    // ------------------------------------------------------------------------
    // Animation Toggle
    // ------------------------------------------------------------------------
    let anim_y = pickers_y + 100.0;
    if cursor.is_over(right_x + 280.0, anim_y, 60.0, 24.0) {
        state.anim_paused = !state.anim_paused;
        return;
    }

    // ------------------------------------------------------------------------
    // Bottom Nav Logic
    // ------------------------------------------------------------------------
    let nav_height = 56.0;
    let nav_y = height - nav_height;
    let item_width = width / 4.0;
    
    if cursor.is_over(0.0, nav_y, width, nav_height) {
        let idx = (cursor.position.unwrap().0 / item_width) as usize;
        if idx < 4 {
            state.selected_bottom_nav = idx;
        }
        return;
    }
}
