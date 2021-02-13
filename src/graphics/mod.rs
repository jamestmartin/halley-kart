pub mod window;

mod vulkan;

use std::default::Default;

use winit::event_loop::EventLoop;

pub use vulkan::{
    DeviceSelection,
    InstanceConfig,
    LayersConfig,
    VulkanConfig,
    VulkanContext,
    VulkanState,
};
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

pub struct GraphicsContext<RpDesc: vulkano::framebuffer::RenderPassDesc> {
    pub event_loop: EventLoop<std::convert::Infallible>,
    pub vulkan: VulkanContext<RpDesc>,
}

pub fn create_graphics_context(
    config: &GraphicsConfig
) -> GraphicsContext<impl vulkano::framebuffer::RenderPassDesc> {
     use window::build_window;

     let event_loop = EventLoop::with_user_event();
     let window = build_window(&event_loop, &config.window)
        .expect("Failed to create window.");

     let vctx = vulkan::setup_vulkan(&config.vulkan, window);

    vctx.surface.window().set_visible(true);

    GraphicsContext { event_loop, vulkan: vctx }
}
