use std::collections::LinkedList;
use std::sync::Arc;
use uuid::Uuid;

use vulkano::device::{Device, Queue};
use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use vulkano::swapchain::Surface;

struct QueueFamilies<'a> {
    physical_device: PhysicalDevice<'a>,
    graphics_family: QueueFamily<'a>,
    present_family: QueueFamily<'a>,
}

impl<'a> QueueFamilies<'a> {
    fn dual_purpose_family(&self) -> bool {
        self.graphics_family == self.present_family
    }
}

fn find_graphics_family<'a>(
    physical_device: &PhysicalDevice<'a>
) -> Option<QueueFamily<'a>> {
    physical_device.queue_families().find(QueueFamily::supports_graphics)
}

fn find_present_family<'a, W>(
    physical_device: &PhysicalDevice<'a>,
    surface: &Surface<W>
) -> Option<QueueFamily<'a>> {
    physical_device.queue_families().find(
        |&q| surface.is_supported(q).unwrap_or(false))
}

fn show_physical_device<'a>(physical_device: &PhysicalDevice<'a>) -> String {
    let name = physical_device.name();
    let uuid = Uuid::from_slice(physical_device.uuid()).unwrap();

    format!("{} (UUID: {})", name, uuid)
}


fn find_queue_families<'a, W>(
    physical_device: PhysicalDevice<'a>,
    surface: &Surface<W>
) -> Option<QueueFamilies<'a>> {
    let graphics_family = match find_graphics_family(&physical_device) {
        Some(x) => x,
        None => {
            log::warn!(
                "Vulkan physical device does not have a graphics queue family \
                 and thus cannot be used: {}",
                show_physical_device(&physical_device)
            );
            return None
        }
    };

    let present_family = match find_present_family(&physical_device, surface) {
        Some(x) => x,
        None => {
            log::warn!(
                "Vulkan physical device does not have a present queue family \
                 and thus cannot be used: {}",
                show_physical_device(&physical_device)
            );
            return None
        }
    };

    Some(QueueFamilies { physical_device, graphics_family, present_family })
}

fn enumerate_usable_physical_devices<'a, W>(
    instance: &'a Arc<Instance>,
    surface: &Surface<W>
) -> LinkedList<QueueFamilies<'a>> {
    use vulkano::device::DeviceExtensions;

    let mut physical_devices = LinkedList::new();
    for physical_device in PhysicalDevice::enumerate(instance) {
        let exts = DeviceExtensions::supported_by_device(physical_device);
        if !exts.khr_swapchain {
            log::warn!(
                "Vulkan physical device does not support swapchains \
                 and thus cannot be used: {}",
                show_physical_device(&physical_device)
            );
            continue;
        }

        if let Some(queue_families) =
            find_queue_families(physical_device, surface) {
                log::info!(
                    "Found eligible Vulkan physical device: {}",
                    show_physical_device(&physical_device)
                );
                physical_devices.push_back(queue_families);
        }
    }

    physical_devices
}

fn select_physical_device<'a, W>(
    instance: &'a Arc<Instance>,
    surface: &Surface<W>
) -> QueueFamilies<'a> {
    let mut physical_devices = enumerate_usable_physical_devices(instance, surface);

    // TODO:
    //  A more sophisticated device selection algorithm + configurability
    //  (also look into the mesa device selection layer).
    //  Fortunately, DRI_PRIME still works, so the user still gets a choice.
    if let Some(device) = physical_devices.pop_front() {
        log::info!(
            "Chose Vulkan physical device: {}",
            show_physical_device(&device.physical_device)
        );
        device
    } else {
        panic!("Failed to find eligible Vulkan physical device.")
    }
}

pub struct Queues {
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub present_queue: Arc<Queue>,
}

impl Queues {
    pub fn dual_purpose_queue(&self) -> bool {
        self.graphics_queue == self.present_queue
    }
}

fn create_device(queue_families: QueueFamilies<'_>) -> Queues {
    use vulkano::device::DeviceExtensions;
    use vulkano::device::Features;

    // Make sure any extensions you add here have been checked for
    // when selecting the physical device.
    let exts = DeviceExtensions {
        khr_swapchain: true,
        .. DeviceExtensions::none()
    };
    // It is illegal to request the same queue family multiple times.
    let fams = if queue_families.dual_purpose_family() {
        vec![(queue_families.graphics_family, 1.0)]
    } else {
        vec![(queue_families.graphics_family, 1.0),
             (queue_families.present_family, 0.5)
        ]
    };

    let (device, mut queues) = Device::new(
        queue_families.physical_device,
        &Features::none(),
        &exts,
        fams
    ).expect("Failed to create Vulkan logical device.");

    let graphics_queue = queues.next().unwrap();
    let present_queue = queues.next().unwrap_or(graphics_queue.clone());

    Queues { device, graphics_queue, present_queue }
}

pub fn setup_device<'a, W>(
    instance: &'a Arc<Instance>,
    surface: &Surface<W>
) -> Queues {
    let physical_device = select_physical_device(instance, surface);
    create_device(physical_device)
}
