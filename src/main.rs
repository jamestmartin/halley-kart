mod graphics;

fn main() {
    stderrlog::new().verbosity(4).init().unwrap();

    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new();

    use winit::window::WindowBuilder;
    let window = WindowBuilder::new()
        // TODO: Make the window resizable
        .with_resizable(false)
        .with_title("Halley Kart")
        .with_decorations(false)
        // TODO: Window icon
        // TODO: Support fullscreen.
        .build(&event_loop)
        .expect("Failed to create window.");

    let instance = graphics::vulkan::create_instance();
    let surface = vulkano_win::create_vk_surface(window, instance.clone()).expect("Failed to create Vulkan surface for window.");
    let queue_families = graphics::vulkan::select_physical_device(&instance, &surface);
    let device = graphics::vulkan::create_device(queue_families);

    event_loop.run(|event, _, control_flow| {
        use winit::event::Event::*;
        use winit::event::WindowEvent::CloseRequested;
        use winit::event_loop::ControlFlow;

        match event {
            WindowEvent { event: CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
                log::info!("Window close requested.");
            },
            LoopDestroyed => {
                log::info!("Goodbye!");
            },
            _ => ()
        }
    });
}
