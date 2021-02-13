use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, ImmutableBuffer},
    command_buffer::{
        AutoCommandBuffer,
        AutoCommandBufferBuilder,
        DynamicState,
        SubpassContents,
        pool::standard::StandardCommandPoolAlloc,
    },
    device::{Device, Queue},
    framebuffer::{
        Framebuffer,
        FramebufferAbstract,
        RenderPass,
        RenderPassDesc,
    },
    image:: SwapchainImage,
    pipeline::viewport::Viewport,
};
use winit::window::Window;

use super::graphics_pipeline::{Gp, Vertex};

type Fb<D> = Framebuffer<Arc<RenderPass<D>>, ((), Arc<SwapchainImage<Window>>)>;

pub fn create_framebuffers<D>(
    rp: &Arc<RenderPass<D>>,
    images: &[Arc<SwapchainImage<Window>>]
)-> Box<[Arc<Fb<D>>]>
where
    D: RenderPassDesc + Send + Sync,
{
    let mut fbs = Vec::new();
    for image in images {
        fbs.push(
            Arc::new(Framebuffer::start(rp.clone())
                     .add(image.clone())
                     .expect("Failed to add attachment to Vulkan framebuffer")
                     .build()
                     .expect("Failed to create Vulkan framebuffer")));
    }
    fbs.into_boxed_slice()
}

fn vertex_buffer(queue: Arc<Queue>) -> Arc<ImmutableBuffer<[Vertex]>> {
    let vertex1 = Vertex { position: [-0.5, -0.5, 0.0] };
    let vertex2 = Vertex { position: [ 0.0,  0.5, 0.0] };
    let vertex3 = Vertex { position: [ 0.5, -0.25, 0.0] };
    ImmutableBuffer::from_iter(
        vec![vertex1, vertex2, vertex3].into_iter(),
        BufferUsage::vertex_buffer(),
        queue
    ).unwrap().0
}

fn build_command_buffer(
    device: Arc<Device>,
    gp: Arc<Gp<impl RenderPassDesc + Send + Sync + 'static>>,
    queue: Arc<Queue>,
    fb: Arc<Fb<impl RenderPassDesc + Send + Sync + 'static>>
) -> AutoCommandBuffer<StandardCommandPoolAlloc> {
    let dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [1280.0, 720.0],
            depth_range: 0.0 .. 1.0,
        }]),
        .. DynamicState::none()
    };

    let mut builder = AutoCommandBufferBuilder::primary(
        device,
        queue.family()
    ).unwrap();

    let vb = vertex_buffer(queue);

    builder
        .begin_render_pass(
            fb as Arc<dyn FramebufferAbstract + Send + Sync>,
            SubpassContents::Inline,
            vec![[0.0, 0.0, 1.0, 1.0].into()]
        )
        .unwrap()
        .draw(gp, &dynamic_state, vb, (), ()).unwrap()
        .end_render_pass().unwrap();

    builder.build().unwrap()
}

pub fn build_command_buffers(
    device: Arc<Device>,
    gp: &Arc<Gp<impl RenderPassDesc + Send + Sync + 'static>>,
    queue: Arc<Queue>,
    fbs: &[Arc<Fb<impl RenderPassDesc + Send + Sync + 'static>>]
) -> Box<[Arc<AutoCommandBuffer<StandardCommandPoolAlloc>>]> {
    let mut out = Vec::new();
    for fb in fbs {
        out.push(Arc::new(build_command_buffer(
            device.clone(),
            gp.clone(),
            queue.clone(),
            fb.clone()
        )));
    }
    out.into_boxed_slice()
}
