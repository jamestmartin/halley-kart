mod device;
mod instance;

use winit::window::Window;

pub use crate::graphics::vulkan::device::Queues;
use crate::graphics::vulkan::device::setup_device;

pub fn setup_vulkan(window: &Window) -> Queues {
    let instance = crate::graphics::vulkan::instance::create_instance();
    let surface = vulkano_win::create_vk_surface(window, instance.clone())
        .expect("Failed to create Vulkan surface for window.");
    setup_device(&instance, &surface)
}
