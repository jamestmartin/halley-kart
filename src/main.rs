fn main() {
    stderrlog::new().verbosity(2).init().unwrap();
    
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
