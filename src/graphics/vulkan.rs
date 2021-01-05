use std::boxed::Box;
use std::sync::Arc;
use std::vec::Vec;

use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::instance::QueueFamily;
use vulkano::swapchain::Surface;

fn application_info<'a>() -> vulkano::instance::ApplicationInfo<'a> {
    use vulkano::instance::ApplicationInfo;
    use vulkano::instance::Version;

    // If you make a fork of this game, you may wish to change the application name,
    // but you should leave the engine name the same.
    ApplicationInfo {
        application_name: Some("Halley Kart".into()),
        application_version: Some(Version { major: 0, minor: 1, patch: 0 }),
        engine_name: Some("Halley Kart".into()),
        engine_version: Some(Version { major: 0, minor: 1, patch: 0 }),
    }
}

fn instance_extensions() -> vulkano::instance::InstanceExtensions {
    use vulkano::instance::InstanceExtensions;

    let required_extensions = vulkano_win::required_extensions();

    let available_extensions = InstanceExtensions::supported_by_core().expect("Failed to list Vulkan instance extensions.");
    let missing_extensions = required_extensions.difference(&available_extensions);
    if missing_extensions != InstanceExtensions::none() {
        panic!("Missing required Vulkan instance extensions.");
    }

    let extensions = available_extensions.intersection(&required_extensions);
    log::debug!("Using Vulkan instance extensions: {:?}", extensions);

    extensions
}

fn instance_layers() -> Box<[Box<str>]> {
    use vulkano::instance::layers_list;

    let desired_layers = ["VK_LAYER_KHRONOS_validation"];

    let mut layers: Vec<Box<str>> = Vec::new();
    for layer in layers_list().expect("Unable to list Vulkan instance layers.") {
        let name = layer.name();
        if desired_layers.contains(&name) {
            log::debug!("Using Vulkan instance layer: {}", name);
            layers.push(String::from(name).into_boxed_str());
        } else {
            log::debug!("Ignoring Vulkan instance layer: {}", name);
        }
    }

    layers.into_boxed_slice()
}

pub fn create_instance() -> Arc<Instance> {
    let exts = vulkano::instance::RawInstanceExtensions::from(&instance_extensions());
    Instance::new(Some(&application_info()), exts, instance_layers().into_iter().map(AsRef::as_ref))
        .expect("Failed to create Vulkan instance.")
}

pub struct QueueFamilies<'a> {
    physical_device: PhysicalDevice<'a>,
    graphics_family: QueueFamily<'a>,
    present_family: QueueFamily<'a>,
}

pub fn select_physical_device<'a, W>(instance: &'a Arc<Instance>, surface: &Surface<W>) -> QueueFamilies<'a> {
    use uuid::Uuid;
    use vulkano::device::DeviceExtensions;

    for physical_device in PhysicalDevice::enumerate(instance) {
        let name = physical_device.name();
        let uuid = Uuid::from_slice(physical_device.uuid()).unwrap();

        let exts = DeviceExtensions::supported_by_device(physical_device);
        if !exts.khr_swapchain {
            log::warn!("Vulkan physical device does not support swapchains (weird!) and thus cannot be used: {} (UUID: {})", name, uuid);
            continue;
        }

        let graphics_family = match physical_device.queue_families().find(|&q| q.supports_graphics()) {
            Some(x) => x,
            None => {
                log::warn!("Vulkan physical device does not have a graphics queue family and thus cannot be used: {} (UUID: {})", name, uuid);
                continue
            }
        };

        let present_family = match physical_device.queue_families().find(|&q| surface.is_supported(q).unwrap_or(false)) {
            Some(x) => x,
            None => {
                log::warn!("Vulkan physical device does not have a present queue family and thus cannot be used: {} (UUID: {})", name, uuid);
                continue
            }
        };

        log::info!("Found eligible Vulkan physical device: {} (UUID: {})", name, uuid);

        return QueueFamilies { physical_device, graphics_family, present_family };
    }

    panic!("Failed to find eligible Vulkan physical device.")
}

pub struct Queues {
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
}

pub fn create_device(queue_families: QueueFamilies<'_>) -> Queues {
    use vulkano::device::DeviceExtensions;
    use vulkano::device::Features;

    // Make sure any extensions you add here have been checked for when selecting the physical device.
    let exts = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
    // It is illegal to request the same queue family multiple times.
    let fams = if queue_families.graphics_family == queue_families.present_family {
        vec![(queue_families.graphics_family, 1.0)]
    } else {
        vec![(queue_families.graphics_family, 1.0), (queue_families.present_family, 0.5)]
    };

    let (device, mut queues) = Device::new(queue_families.physical_device, &Features::none(), &exts, fams)
        .expect("Failed to create Vulkan logical device.");

    let graphics_queue = queues.next().unwrap();
    let present_queue = queues.next().unwrap_or(graphics_queue.clone());

    Queues { device, graphics_queue, present_queue }
}
