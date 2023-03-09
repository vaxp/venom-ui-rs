//! VenomUI Widgets - Widget system and layout engine
//!
//! This crate provides:
//! - Widget trait and core widget types
//! - Layout system (Row, Column, Stack)
//! - Basic widgets (Container, Text, Button, etc.)
//! - Form widgets (Checkbox, Radio, Switch, Slider, TextField)
//! - Progress indicators (ProgressBar, ProgressCircle)
//! - Scrolling (ScrollView)
//! - Dialogs (Dialog, Snackbar)
//! - Event handling (Shell, EventStatus, InteractiveWidget)
//!
//! Inspired by Flutter's widget system and Iced's event pattern.

// Note: We use deny instead of forbid to allow the bloc_widgets module
// to implement Send+Sync for thread-safe state management widgets.
#![deny(unsafe_code)]
#![warn(missing_docs)]

mod widget;
mod element;
mod layout;
mod basic;
mod text;
mod button;
mod gesture;
mod image;

// Form widgets
mod event;
mod checkbox;
mod radio;
mod switch;
mod slider;
mod progress;

// Advanced widgets
mod scrollview;
mod textfield;
mod dialog;

// Event system
mod interactive;
mod shell;

// State management widgets
mod bloc_widgets;

// Layout helpers
mod layout_helpers;

// Cards and decorative widgets
mod cards;

// Tooltip
mod tooltip;

// Navigation
mod navigation;

// Lists
mod lists;

// Pickers
mod pickers;

// Animation
mod animation;

pub use widget::*;
pub use element::*;
pub use layout::*;
pub use basic::*;
pub use text::*;
pub use button::*;
pub use gesture::*;
pub use image::*;

// Form exports
pub use event::*;
pub use checkbox::*;
pub use radio::*;
pub use switch::*;
pub use slider::*;
pub use progress::*;

// Advanced exports
pub use scrollview::*;
pub use textfield::*;
pub use dialog::*;

// Event system exports
pub use interactive::*;
pub use shell::*;

// State management widget exports
pub use bloc_widgets::*;

// Layout helpers exports
pub use layout_helpers::*;

// Cards exports
pub use cards::*;

// Tooltip exports
pub use tooltip::*;

// Navigation exports
pub use navigation::*;

// Lists exports
pub use lists::*;

// Pickers exports
pub use pickers::*;

// Animation exports
pub use animation::*;

// Re-export commonly used types
pub use venom_core::{
    BoxConstraints, Size, Offset, Insets, Color, 
    MainAxisAlignment, CrossAxisAlignment, Alignment,
};

// Re-export state management from venom-core
pub use venom_core::{
    // Subscription system
    StateStream, SubscriptionHandle, SubscriptionId,
    // Cubit
    CubitCore, SimpleCubit, SharedCubit, Cubit,
    // Bloc
    StatefulBloc, BlocCore, Bloc, EventTransformer, AsyncResult,
    // Selectors
    Selector, SelectorEq, ComposedSelector, Selector2, Selector3,
    // Provider
    ProviderContext, MultiProviderBuilder, RepositoryProvider, ScopedProvider,
    BlocBuilderConfig, BlocListenerConfig,
    // DevTools
    DevTools, DevToolsConfig, BlocObserver, ConsoleObserver, SilentObserver,
    StateHistory, HistoryEntry, DebuggableCubit,
    // Async
    AsyncCubit, ThreadedBloc, CallbackBloc, DebouncedCubit,
    // Hydration
    Hydratable, HydrationStorage, HydrationError,
    MemoryStorage, FileStorage, HydratedCubit,
    // Thread-safe state management
    SyncStateStream, SyncSubscriptionId, SyncSubscriptionHandle,
    SyncCubitCore, SyncCubit, SyncSimpleCubit, SyncBloc,
};
