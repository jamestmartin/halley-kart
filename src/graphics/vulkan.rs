mod device;
mod instance;

use winit::window::Window;

pub use device::DeviceSelection;
use device::{
    DeviceExt,
    PhysicalDeviceExt,
    select_physical_device,
    show_physical_device,
};
use instance::{
    QueriedInstanceFeatures,
    InstanceFeatures,
    InstanceLayers,
    create_instance,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VulkanConfig {
    pub device: DeviceSelection,
    pub instance: InstanceConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstanceConfig {
    pub layers: LayersConfig,
}

impl InstanceConfig {
    fn into_instance_features(&self) -> InstanceFeatures {
        InstanceFeatures {
            layers: self.layers.into_instance_layers(),
            .. InstanceFeatures::none()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayersConfig {
    pub mesa_device_select: bool,
    pub mesa_overlay: bool,
    pub khronos_validation: bool,
}

impl LayersConfig {
    fn into_instance_layers(&self) -> InstanceLayers {
        InstanceLayers {
            mesa_device_select: self.mesa_device_select,
            mesa_overlay: self.mesa_overlay,
            khronos_validation: self.khronos_validation,
            .. InstanceLayers::none()
        }
    }
}

impl Default for LayersConfig {
    fn default() -> Self {
        Self {
            mesa_device_select: true,
            mesa_overlay: false,
            khronos_validation: true,
        }
    }
}

pub fn setup_vulkan(config: &VulkanConfig, window: &Window) {
    let available_features = match QueriedInstanceFeatures::query() {
        Ok(x) => x,
        Err(missing) => {
            panic!("Missing required Vulkan instance features: {:?}", missing)
        }
    };
    let requested_features = config.instance.into_instance_features();

    let instance = create_instance(&available_features, &requested_features);

    let surface = vulkano_win::create_vk_surface(window, instance.clone())
        .expect("Failed to create Vulkan surface for window.");

    let physical_devices = PhysicalDeviceExt::enumerate(&instance, &surface);

    let physical_device =
        select_physical_device(config.device, physical_devices);
    log::info!(
        "Using Vulkan physical device: {}",
        show_physical_device(physical_device.physical_device())
    );

    let _device = DeviceExt::create(physical_device);
}
