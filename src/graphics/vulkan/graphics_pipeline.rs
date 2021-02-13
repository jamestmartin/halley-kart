use std::sync::Arc;

use vulkano::{
    descriptor::pipeline_layout::PipelineLayoutAbstract,
    device::Device,
    framebuffer::{RenderPass, RenderPassDesc},
    image::SwapchainImage,
    pipeline::{
        GraphicsPipeline,
        vertex::SingleBufferDefinition,
    },
    swapchain::{Surface, Swapchain},
};
use winit::window::Window;

use super::device::DeviceExt;

pub fn create_swapchain(
    ext: &DeviceExt,
    surface: Arc<Surface<Window>>
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
    use vulkano::image::ImageUsage;
    use vulkano::swapchain::{ColorSpace, CompositeAlpha, FullscreenExclusive, PresentMode, SurfaceTransform};
    use vulkano::sync::SharingMode;

    log::trace!("Creating Vulkan swapchain.");
    let device = &ext.device;
    let caps = surface.capabilities(device.physical_device()).unwrap();
    let dimensions = caps.current_extent.unwrap_or_else(|| surface.window().inner_size().into());
    // At least two images should be guaranteed because the FIFO present mode is guaranteed.
    let num_images = caps.min_image_count.max(2);
    // TODO: Deal with color formats which are potentially different.
    let format = caps.supported_formats[0].0;

    let queues = &ext.queues;
    let sharing_mode = if queues.graphics_queue != queues.present_queue {
        SharingMode::from(vec![&queues.graphics_queue, &queues.present_queue].as_slice())
    } else {
        SharingMode::from(&queues.graphics_queue)
    };

    Swapchain::new(device.clone(), surface, num_images, format, dimensions, 1,
                   ImageUsage::color_attachment(), sharing_mode, SurfaceTransform::Identity, CompositeAlpha::Opaque,
                   // TODO: Support fullscreen stuff properly.
                   // TODO: Support alternative presentmodes.
                   PresentMode::Fifo, FullscreenExclusive::AppControlled, false, ColorSpace::SrgbNonLinear)
        .expect("Failed to create Vulkan swapchain.")
}

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position);

pub type Gp<D> = GraphicsPipeline<SingleBufferDefinition<Vertex>,
                                  Box<dyn PipelineLayoutAbstract + Send + Sync>,
                                  Arc<RenderPass<D>>>;

pub fn create_pipeline(
    device: &Arc<Device>,
    swapchain: &Arc<Swapchain<Window>>
) -> Arc<Gp<impl RenderPassDesc>> {
    use vulkano::framebuffer::Subpass;
    use super::shaders::{fragment, vertex};

    let fs = fragment::Shader::load(device.clone()).expect("Failed to create fragment shader module.");
    let vs = vertex::Shader::load(device.clone()).expect("Failed to create vertex shader module.");

    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());

    // TODO: Antialiasing, culling, investigate more complex setting/render passes
    Arc::new(GraphicsPipeline::start()
             .vertex_input_single_buffer::<Vertex>()
             .triangle_list()
             .vertex_shader(vs.main_entry_point(), ())
             .fragment_shader(fs.main_entry_point(), ())
             .viewports_dynamic_scissors_irrelevant(1)
             .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
             .build(device.clone())
             .expect("Failed to create Vulkan graphics pipeline"))
}
