pub mod window;

mod vulkan;

use std::default::Default;

use winit::event_loop::EventLoop;
use winit::window::Window;

pub use vulkan::{DeviceSelection, InstanceConfig, LayersConfig, VulkanConfig};
pub use window::{
    MonitorConfig,
    VideoModeConfig,
    WindowedConfig,
    WindowConfig,
    WindowMode
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphicsConfig {
    pub window: WindowConfig,
    pub vulkan: VulkanConfig,
}

pub struct GraphicsContext {
    pub event_loop: EventLoop<()>,
    pub window: Window,
}

pub fn setup_graphics(config: &GraphicsConfig) -> GraphicsContext {
    use window::build_window;

    let event_loop = EventLoop::new();
    let window = build_window(&event_loop, &config.window)
        .expect("Failed to create window.");

    vulkan::setup_vulkan(&config.vulkan, &window);

    window.set_visible(true);

    GraphicsContext { event_loop, window }
}
