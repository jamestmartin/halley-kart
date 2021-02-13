mod audio;
mod config;
mod graphics;

use std::cell::Cell;

use audio::{AudioConfig, AudioContext};
use graphics::{
    GraphicsConfig,
    GraphicsContext,
    VulkanState,
    create_graphics_context,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub audio: AudioConfig,
    pub graphics: GraphicsConfig,
}

struct Context<RpDesc: vulkano::framebuffer::RenderPassDesc> {
    audio: AudioContext,
    graphics: GraphicsContext<RpDesc>,
}

fn create_context(
    config: &Config
) -> Context<impl vulkano::framebuffer::RenderPassDesc> {
    let audio = AudioContext::create(&config.audio).expect("Failed to set up audio");
    let graphics = create_graphics_context(&config.graphics);
    Context { audio, graphics }
}

fn main() {
    stderrlog::new().verbosity(3).init().unwrap();

    let config = config::read_config();
    let context = create_context(&config);
    let GraphicsContext { event_loop, vulkan } = context.graphics;

    let vk_state = Cell::new(Some(VulkanState::create(vulkan)));

    // FIXME:
    //   If the window variable goes out of scope,
    //   there is no window to receive a close request
    //   and this loop never terminates.
    //   Make sure that this loop always terminates
    //   if the window ceases to exist.
    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event::*;
        use winit::event::WindowEvent::CloseRequested;
        use winit::event_loop::ControlFlow;

        match event {
            WindowEvent { event: CloseRequested, .. } => {
                // FIXME: Dependent on below
                let vkst = vk_state.take().unwrap();
                vkst.context.surface.window().set_visible(false);
                vk_state.set(Some(vkst));
                *control_flow = ControlFlow::Exit;
                log::trace!("Window close requested.");
            },
            LoopDestroyed => {
                log::info!("Goodbye!");
            },
            RedrawEventsCleared => {
                // FIXME: What an awful borrow checker workaround.
                vk_state.set(Some(vk_state.take().unwrap().draw()));
            },
            _ => ()
        }
    });
}
