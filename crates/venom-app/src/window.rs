//! Window management with winit
//!
//! Provides cross-platform window creation and event handling.

use std::num::NonZeroU32;
use std::rc::Rc;
use std::cell::RefCell;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};
use winit::dpi::LogicalSize;

use venom_core::{Color, Rect, Point, Size};
use venom_render::{SoftwareBackend, PaintCanvas, Paint};

// ============================================================================
// REDRAW TYPES
// ============================================================================

/// Type of redraw requested
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RedrawRequest {
    /// No redraw needed
    None,
    /// Redraw specific region
    Partial(Rect),
    /// Redraw entire window
    Full,
}

impl RedrawRequest {
    /// Merge with another request
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Full, _) | (_, Self::Full) => Self::Full,
            (Self::None, other) => other,
            (other, Self::None) => other,
            (Self::Partial(r1), Self::Partial(r2)) => Self::Partial(r1.union(r2)),
        }
    }
}

// ============================================================================
// APP CONFIG
// ============================================================================

/// Configuration for creating an application window
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Window title
    pub title: String,
    /// Initial width in logical pixels
    pub width: u32,
    /// Initial height in logical pixels
    pub height: u32,
    /// Whether the window is resizable
    pub resizable: bool,
    /// Whether to enable VSync
    pub vsync: bool,
    /// Background color
    pub background: Color,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "VenomUI App".into(),
            width: 800,
            height: 600,
            resizable: true,
            vsync: true,
            background: Color::hex("#1a1a2e"),
        }
    }
}

impl AppConfig {
    /// Create a new config with title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set window size
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }
}

// ============================================================================
// RENDER CONTEXT
// ============================================================================

/// Context passed to the render callback
pub struct RenderContext {
    /// Current window width
    pub width: u32,
    /// Current window height
    pub height: u32,
    /// Mouse position (if available)
    pub mouse_position: Option<Point>,
    /// Whether mouse button is pressed
    pub mouse_pressed: bool,
    /// Time since last frame (seconds)
    pub delta_time: f32,
    /// Frame number
    pub frame: u64,
    /// Request a redraw for the next frame (animation)
    pub(crate) request_redraw: RefCell<RedrawRequest>,
}

impl RenderContext {
    /// Get the window size
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    /// Request a redraw for the next frame
    /// 
    /// If `rect` is provided, only that region will be redrawn.
    /// If `None`, the entire window will be redrawn.
    pub fn request_redraw(&self, rect: Option<Rect>) {
        let mut current = self.request_redraw.borrow_mut();
        let request = match rect {
            Some(r) => RedrawRequest::Partial(r),
            None => RedrawRequest::Full,
        };
        *current = current.merge(request);
    }
}

// ============================================================================
// APP STATE
// ============================================================================

/// Internal app state during event loop
struct AppState<F> 
where 
    F: FnMut(&mut SoftwareBackend, &RenderContext),
{
    /// Window configuration
    config: AppConfig,
    /// The window (created on resume)
    window: Option<Rc<Window>>,
    /// Software buffer for presenting
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    /// Render backend
    backend: Option<SoftwareBackend>,
    /// User render callback
    render_fn: F,
    /// Current mouse position
    mouse_pos: Option<Point>,
    /// Mouse button state
    mouse_pressed: bool,
    /// Frame counter
    frame: u64,
    /// Accumulated damage for the next frame
    damage: RedrawRequest,
}

impl<F> AppState<F>
where
    F: FnMut(&mut SoftwareBackend, &RenderContext),
{
    fn new(config: AppConfig, render_fn: F) -> Self {
        Self {
            config,
            window: None,
            surface: None,
            backend: None,
            render_fn,
            mouse_pos: None,
            mouse_pressed: false,
            frame: 0,
            damage: RedrawRequest::Full, // Start with full redraw
        }
    }
}

impl<F> ApplicationHandler for AppState<F>
where
    F: FnMut(&mut SoftwareBackend, &RenderContext),
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        let window_attrs = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(self.config.resizable);

        let window = Rc::new(
            event_loop.create_window(window_attrs)
                .expect("Failed to create window")
        );

        // Create software surface
        let context = softbuffer::Context::new(window.clone())
            .expect("Failed to create softbuffer context");
        let surface = softbuffer::Surface::new(&context, window.clone())
            .expect("Failed to create surface");

        // Create backend
        let size = window.inner_size();
        let backend = SoftwareBackend::new(size.width, size.height);

        self.window = Some(window);
        self.surface = Some(surface);
        self.backend = Some(backend);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    // Resize backend
                    self.backend = Some(SoftwareBackend::new(size.width, size.height));
                    
                    // Resize surface
                    if let Some(surface) = &mut self.surface {
                        surface.resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        ).expect("Failed to resize surface");
                    }
                    
                    // Request full redraw
                    self.damage = RedrawRequest::Full;
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Point::new(position.x as f32, position.y as f32);
                self.mouse_pos = Some(new_pos);
                
                // For safety with immediate mode widgets (hover states), we currently 
                // trigger a full redraw on mouse move.
                // In a more advanced system, we would hit-test widgets and only damage changed ones.
                self.damage = RedrawRequest::Full;
                
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorLeft { .. } => {
                self.mouse_pos = None;
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    self.mouse_pressed = state == winit::event::ElementState::Pressed;
                    
                    // Just damage everything on click for simplicity (or we could propagate click damage)
                    // For now, let's assume UI might change significantly on click
                    self.damage = RedrawRequest::Full;
                    
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                // If no damage, skip
                if self.damage == RedrawRequest::None {
                    return;
                }
                
                self.frame += 1;
                
                let Some(window) = &self.window else { return };
                let Some(surface) = &mut self.surface else { return };
                let Some(backend) = &mut self.backend else { return };

                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }

                // Create render context
                let ctx = RenderContext {
                    width: size.width,
                    height: size.height,
                    mouse_position: self.mouse_pos,
                    mouse_pressed: self.mouse_pressed,
                    delta_time: 1.0 / 60.0, // TODO: actual timing
                    frame: self.frame,
                    request_redraw: RefCell::new(RedrawRequest::None),
                };

                // Handle clear and clip based on damage
                backend.save();
                match self.damage {
                    RedrawRequest::Full => {
                        backend.clear(self.config.background);
                    }
                    RedrawRequest::Partial(rect) => {
                        backend.set_clip_rect(Some(rect));
                        // Clear only the dirty rect
                        backend.draw_rect(
                            Rect::new(0.0, 0.0, size.width as f32, size.height as f32), 
                            &Paint::fill(self.config.background)
                        );
                    }
                    RedrawRequest::None => unreachable!(),
                }

                // Call user render function
                (self.render_fn)(backend, &ctx);
                
                backend.restore();

                // Check for next frame requests
                self.damage = match *ctx.request_redraw.borrow() {
                    RedrawRequest::None => RedrawRequest::None,
                    req => {
                        window.request_redraw();
                        req
                    }
                };

                // Present
                let mut buffer = surface.buffer_mut()
                    .expect("Failed to get buffer");

                // Convert RGBA to native format (ARGB or XRGB depending on platform)
                let pixels = backend.pixels();
                for (i, pixel) in buffer.iter_mut().enumerate() {
                    let base = i * 4;
                    if base + 3 < pixels.len() {
                        let r = pixels[base] as u32;
                        let g = pixels[base + 1] as u32;
                        let b = pixels[base + 2] as u32;
                        *pixel = (r << 16) | (g << 8) | b;
                    }
                }

                buffer.present().expect("Failed to present");
            }

            _ => {}
        }
    }
}

// ============================================================================
// APP
// ============================================================================

/// Main application entry point
/// 
/// # Example
/// 
/// ```ignore
/// use venom_app::{App, AppConfig};
/// use venom_render::{Paint, PaintCanvas};
/// use venom_core::{Color, Rect};
/// 
/// App::run(AppConfig::new("Demo"), |canvas, ctx| {
///     canvas.draw_rect(
///         Rect::new(100.0, 100.0, 200.0, 100.0),
///         &Paint::fill(Color::RED)
///     );
/// });
/// ```
pub struct App;

impl App {
    /// Run the application with a render callback
    /// 
    /// The callback receives a mutable reference to the render backend
    /// and a render context with frame information.
    pub fn run<F>(config: AppConfig, render_fn: F)
    where
        F: FnMut(&mut SoftwareBackend, &RenderContext) + 'static,
    {
        let event_loop = EventLoop::new()
            .expect("Failed to create event loop");
        
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut state = AppState::new(config, render_fn);
        
        event_loop.run_app(&mut state)
            .expect("Event loop error");
    }

    /// Run with default config
    pub fn run_default<F>(render_fn: F)
    where
        F: FnMut(&mut SoftwareBackend, &RenderContext) + 'static,
    {
        Self::run(AppConfig::default(), render_fn);
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Quick way to run a simple demo
/// 
/// ```ignore
/// venom_app::run_demo();
/// ```
pub fn run_demo() {
    App::run(
        AppConfig::new("VenomUI Demo")
            .size(800, 600)
            .background(Color::hex("#0f0f1a")),
        |canvas, ctx| {
            // Draw some shapes for demo
            let center_x = ctx.width as f32 / 2.0;
            let center_y = ctx.height as f32 / 2.0;

            // Animated circle
            let time = ctx.frame as f32 * 0.02;
            let radius = 50.0 + (time.sin() * 20.0);
            
            canvas.draw_circle(
                Point::new(center_x, center_y),
                radius,
                &Paint::fill(Color::hex("#6366f1")),
            );

            // Draw rect
            canvas.draw_rounded_rect(
                Rect::new(50.0, 50.0, 150.0, 80.0),
                venom_core::BorderRadius::all(12.0),
                &Paint::fill(Color::hex("#22c55e")),
            );

            // Draw mouse indicator
            if let Some(mouse) = ctx.mouse_position {
                canvas.draw_circle(
                    mouse,
                    10.0,
                    &Paint::fill(Color::hex("#f97316")),
                );
            }

            // Request animation frame for the circle area
            // Radius is max ~70.0
            let damage = Rect::new(center_x - 80.0, center_y - 80.0, 160.0, 160.0);
            ctx.request_redraw(Some(damage));
        },
    );
}
