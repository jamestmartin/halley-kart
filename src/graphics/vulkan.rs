mod device;
mod instance;

use winit::window::Window;

pub fn setup_vulkan(window: &Window) {
    use crate::graphics::vulkan::device::{
        DeviceExt,
        PhysicalDeviceExt,
        select_physical_device
    };
    use crate::graphics::vulkan::instance::{
        QueriedInstanceFeatures,
        InstanceFeatures,
        InstanceLayers,
        create_instance,
    };

    let available_features = match QueriedInstanceFeatures::query() {
        Ok(x) => x,
        Err(missing) => {
            panic!("Missing required Vulkan instance features: {:?}", missing)
        }
    };
    let requested_features = InstanceFeatures {
        layers: InstanceLayers {
            khronos_validation: true,
            .. InstanceLayers::none()
        },
        .. InstanceFeatures::none()
    };

    let instance = create_instance(&available_features, &requested_features);

    let surface = vulkano_win::create_vk_surface(window, instance.clone())
        .expect("Failed to create Vulkan surface for window.");

    let physical_devices = PhysicalDeviceExt::enumerate(&instance, &surface);
    let _device = DeviceExt::create(select_physical_device(physical_devices));
}
