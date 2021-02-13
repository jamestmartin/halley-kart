mod device;
mod drawing;
mod graphics_pipeline;
mod instance;
mod shaders;

use std::sync::Arc;

use vulkano::{
    command_buffer::{
        AutoCommandBuffer,
        pool::standard::StandardCommandPoolAlloc
    },
    swapchain::Surface,
    sync::GpuFuture,
};
use winit::window::Window;

pub use device::DeviceSelection;
use device::{
    DeviceExt,
    PhysicalDeviceExt,
    select_physical_device,
    show_physical_device,
};
use drawing::{
    build_command_buffers,
    create_framebuffers,
};
use graphics_pipeline::{
    create_swapchain,
    create_pipeline,
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


use vulkano::{
    framebuffer::{Framebuffer, RenderPass, RenderPassDesc},
    image::SwapchainImage,
    swapchain::Swapchain,
};
pub struct VulkanContext<RpDesc: RenderPassDesc> {
    ext: DeviceExt,
    pub surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    framebuffers:
    Box<[Arc<Framebuffer<
            Arc<RenderPass<RpDesc>>,
            ((), Arc<SwapchainImage<winit::window::Window>>)
            >>]>,
    command_buffers:
    Box<[Arc<AutoCommandBuffer<StandardCommandPoolAlloc>>]>
}

pub fn setup_vulkan(
    config: &VulkanConfig,
    window: Window
) -> VulkanContext<impl RenderPassDesc> {
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

    let physical_devices =
        PhysicalDeviceExt::enumerate(&instance, surface.as_ref());

    let physical_device =
        select_physical_device(config.device, physical_devices);
    log::info!(
        "Using Vulkan physical device: {}",
        show_physical_device(physical_device.physical_device())
    );

    let ext = DeviceExt::create(physical_device);

    let (swapchain, images) = create_swapchain(&ext, surface.clone());
    let pipeline = create_pipeline(&ext.device, &swapchain);
    let fbs = create_framebuffers(pipeline.render_pass(), images.as_slice());
    let command_buffers = build_command_buffers(
        ext.device.clone(),
        &pipeline,
        ext.queues.graphics_queue.clone(),
        fbs.as_ref()
    );

    VulkanContext {
        ext,
        surface,
        swapchain,
        images,
        framebuffers: fbs,
        command_buffers,
    }
}

pub struct VulkanState<RpDesc: RenderPassDesc> {
    pub context: VulkanContext<RpDesc>,
    pub previous_frame_end: Box<dyn GpuFuture>,
}

impl<T: RenderPassDesc> VulkanState<T> {
    pub fn create(ctx: VulkanContext<T>) -> Self {
        let previous_frame_end =
            vulkano::sync::now(ctx.ext.device.clone()).boxed();
        Self { context: ctx, previous_frame_end }
    }

    pub fn draw(mut self) -> Self {
        self.previous_frame_end.cleanup_finished();

        // FIXME:
        //   Re-create the swapchain when the window is resized
        //   instead of just crashing.
        let (image_num, _, acquire_future) =
            vulkano::swapchain::acquire_next_image(
                self.context.swapchain.clone(),
                None
            ).expect("Failed to acquire next image");
        let command_buffer = self.context.command_buffers[image_num].clone();

        // FIXME:
        //   I don't want to block the main/event thread
        //   waiting for the previous frame to finish processing.
        //   However, joining on it is not sufficient because
        //   `then_execute` expects the command_buffer to be free *now*
        //   even if it *would* be free after the previous frame is finished,
        //   and if I make the command buffer concurrent this function
        //   periodically deadlocks for unknown (but presumably related) reasons.
        //   (Note that when a frame is finished *processing* is different
        //   from when a frame has been *presented*. This just waits for
        //   the previous frame to be finished *processing*, but in e.g. mailbox
        //   mode we would still start the next frame before the
        //   previous was presented).
        //
        //   I think the best solution here would most likely be to
        //   simply move the GPU command loop to a separate
        //   blocking, async task.
        std::mem::drop(self.previous_frame_end);

        let future = acquire_future
            //.join(self.previous_frame_end)
            .then_execute(
                self.context.ext.queues.graphics_queue.clone(),
                command_buffer
            )
            .unwrap()
            .then_swapchain_present(
                self.context.ext.queues.present_queue.clone(),
                self.context.swapchain.clone(),
                image_num
            )
            .then_signal_fence_and_flush()
            .unwrap();

        Self {
            context: self.context,
            previous_frame_end: future.boxed(),
        }
    }
}
