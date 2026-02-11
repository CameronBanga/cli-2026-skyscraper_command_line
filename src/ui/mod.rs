pub mod composer;
pub mod login;
pub mod post_widget;
pub mod profile;
pub mod statusbar;
pub mod tabs;
pub mod thread;
pub mod timeline;

use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::action::Action;

/// Trait for **modal components** (e.g. `LoginForm`, `Composer`) that own state
/// and intercept keyboard input while active.
///
/// Non-modal views (timeline, thread, profile) are rendered by stateless
/// `draw_*()` functions instead — they read shared `App` state and do not
/// intercept keys, so they do not need to implement this trait.
pub trait Component {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action>;
    fn update(&mut self, action: &Action);
    fn draw(&self, frame: &mut Frame, area: ratatui::prelude::Rect);
}
