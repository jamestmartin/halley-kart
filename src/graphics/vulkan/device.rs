use std::sync::Arc;
use uuid::Uuid;

use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use vulkano::swapchain::Surface;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DeviceSelection {
    Auto,
    Best,
    Uuid(Uuid),
}

impl Default for DeviceSelection {
    fn default() -> Self {
        Self::Best
    }
}

#[derive(Copy, Clone)]
struct QueueFamilies<'a> {
    graphics_family: QueueFamily<'a>,
    present_family: QueueFamily<'a>,
}

pub fn show_physical_device<'a>(physical_device: &PhysicalDevice<'a>) -> String {
    let name = physical_device.name();
    let uuid = Uuid::from_slice(physical_device.uuid()).unwrap();

    format!("{} (UUID: {})", name, uuid)
}

pub enum MissingQueueFamily {
    MissingGraphicsFamily,
    MissingPresentFamily,
}
use MissingQueueFamily::*;

impl MissingQueueFamily {
    fn log_warn<'a>(&self, physical_device: &PhysicalDevice<'a>) {
        let disp = show_physical_device(physical_device);
        match self {
            MissingGraphicsFamily =>
                log::warn!("Cannot use Vulkan physical device because \
                            it is missing a graphics queue family: {}",
                           disp),
            MissingPresentFamily =>
                log::warn!("Cannot use Vulkan physical device because \
                            it is missing a present queue family: {}",
                           disp),
        };
    }
}

pub enum MissingExtension {
    MissingSwapchain
}
use MissingExtension::*;

impl MissingExtension {
    fn log_warn<'a>(&self, physical_device: &PhysicalDevice<'a>) {
        let disp = show_physical_device(physical_device);
        match self {
            MissingSwapchain =>
                log::warn!("Cannot use Vulkan physical device because \
                            it does not support the swapchain extension: {}",
                           disp),
        }
    }
}


pub enum MissingFeature {
    MissingQueueFamily(MissingQueueFamily),
    MissingExtension(MissingExtension),
}
use MissingFeature::*;

impl MissingFeature {
    fn log_warn<'a>(&self, physical_device: &PhysicalDevice<'a>) {
        match self {
            MissingQueueFamily(err) => err.log_warn(physical_device),
            MissingExtension(err) => err.log_warn(physical_device),
        }
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

impl<'a> QueueFamilies<'a> {
    fn find_queue_families<W>(
        physical_device: &PhysicalDevice<'a>,
        surface: &Surface<W>
    ) -> Result<Self, MissingQueueFamily> {
        let graphics_family = find_graphics_family(physical_device)
            .ok_or(MissingGraphicsFamily)?;
        let present_family = find_present_family(physical_device, surface)
            .ok_or(MissingPresentFamily)?;
        Ok(QueueFamilies { graphics_family, present_family })
    }

    fn priorities(&self) -> Box<[(QueueFamily<'a>, f32)]> {
        if self.dual_purpose_family() {
            vec![(self.graphics_family, 1.0)]
        } else {
            vec![(self.graphics_family, 1.0), (self.present_family, 1.0)]
        }.into_boxed_slice()
    }

    fn dual_purpose_family(&self) -> bool {
        self.graphics_family == self.present_family
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PhysicalDeviceFeatures {
    pub extensions: DeviceExtensions,
    pub features: Features,
}

impl PhysicalDeviceFeatures {
    fn query(physical_device: &PhysicalDevice<'_>) -> Self {
        let extensions = DeviceExtensions::supported_by_device(*physical_device);
        let features = physical_device.supported_features().clone();
        Self { extensions, features }
    }
}

#[derive(Clone)]
pub struct PhysicalDeviceExt<'a> {
    physical_device: PhysicalDevice<'a>,
    queue_families: QueueFamilies<'a>,
    features: PhysicalDeviceFeatures,
}

impl<'a> PhysicalDeviceExt<'a> {
    fn query<W>(
        physical_device: PhysicalDevice<'a>,
        surface: &Surface<W>
    ) -> Result<Self, MissingQueueFamily> {
        let queue_families =
            QueueFamilies::find_queue_families(&physical_device, surface)?;
        let features = PhysicalDeviceFeatures::query(&physical_device);
        Ok(Self { physical_device, queue_families, features })
    }

    pub fn enumerate<W>(
        instance: &'a Arc<Instance>,
        surface: &Surface<W>
    ) -> Box<[Self]> {
        let mut physical_devices = Vec::new();
        for physical_device in PhysicalDevice::enumerate(instance) {
            match Self::query(physical_device, surface) {
                Ok(x) => {
                    if !x.features.extensions.khr_swapchain {
                        MissingSwapchain.log_warn(&physical_device);
                        continue;
                    }
                    physical_devices.push(x);
                },
                Err(err) => err.log_warn(&physical_device),
            }
        }

        for d in physical_devices.iter() {
            log::info!(
                "Found usable Vulkan physical device: {}",
                show_physical_device(&d.physical_device)
            );
        }

        physical_devices.into_boxed_slice()
    }

    pub fn physical_device<'b>(&'b self) -> &'b PhysicalDevice<'a> {
        &self.physical_device
    }
}

pub fn select_physical_device<'a>(
    config: DeviceSelection,
    physical_devices: Box<[PhysicalDeviceExt<'a>]>
) -> PhysicalDeviceExt<'a> {
    use vulkano::instance::PhysicalDeviceType;

    if let DeviceSelection::Uuid(uuid) = config {
        if let Some(ext) = physical_devices.iter().find(
            |ext| Uuid::from_slice(ext.physical_device.uuid()).unwrap() == uuid
        ) {
            return ext.clone();
        }
    }

    if DeviceSelection::Best == config {
        if let Some(ext) = physical_devices.iter().find(
            |ext| ext.physical_device.ty() == PhysicalDeviceType::DiscreteGpu
        ) {
            return ext.clone();
        }

        if let Some(ext) = physical_devices.iter().find(
            |ext| ext.physical_device.ty() == PhysicalDeviceType::IntegratedGpu
        ) {
            return ext.clone();
        }
    }

    if let Some(ext) = physical_devices.first() {
        return ext.clone();
    }

    panic!("Failed to find eligible Vulkan physical device.")
}

struct Queues {
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
}

pub struct DeviceExt {
    device: Arc<Device>,
    queues: Queues,
}

impl DeviceExt {
    pub fn create(physical_device: PhysicalDeviceExt<'_>) -> Self {
        // Make sure any extensions you add here have been checked for
        // when selecting the physical device.
        let exts = DeviceExtensions {
            khr_swapchain: true,
            .. DeviceExtensions::none()
        };
        // It is illegal to request the same queue family multiple times.
        let fams = physical_device.queue_families.priorities();

        let (device, mut queues) = Device::new(
            physical_device.physical_device,
            &Features::none(),
            &exts,
            fams.into_iter().map(|&q| q)
        ).expect("Failed to create Vulkan logical device.");

        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or(graphics_queue.clone());

        Self { device, queues: Queues { graphics_queue, present_queue } }
    }

    fn device<'a>(&'a self) -> &'a Device {
        &self.device
    }
}

impl Queues {
    pub fn dual_purpose_queue(&self) -> bool {
        self.graphics_queue == self.present_queue
    }
}
