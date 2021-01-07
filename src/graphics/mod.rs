pub mod window;

mod vulkan;

use std::default::Default;

use winit::event_loop::EventLoop;
use winit::window::Window;

use window::*;

pub struct GraphicsContext {
    pub event_loop: EventLoop<()>,
    pub window: Window,
}

pub fn setup_graphics() -> GraphicsContext {
    let event_loop = EventLoop::new();
    let window = build_window(&event_loop, &WindowConfig::default())
        .expect("Failed to create window.");

    vulkan::setup_vulkan(&window);

    window.set_visible(true);

    GraphicsContext { event_loop, window }
}
