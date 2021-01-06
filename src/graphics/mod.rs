mod vulkan;

use winit::event_loop::EventLoop;
use winit::window::Window;

pub struct GraphicsContext {
    pub event_loop: EventLoop<()>,
    pub window: Window,
}

pub fn setup_graphics() -> GraphicsContext {
    let event_loop = EventLoop::new();

    use winit::window::WindowBuilder;
    let window = WindowBuilder::new()
        // TODO: Make the window resizable
        .with_resizable(false)
        .with_title("Halley Kart")
        .with_decorations(false)
        // TODO: Window icon
        // TODO: Support fullscreen.
        .build(&event_loop)
        .expect("Failed to create window.");

    crate::graphics::vulkan::setup_vulkan(&window);

    GraphicsContext { event_loop, window }
}
