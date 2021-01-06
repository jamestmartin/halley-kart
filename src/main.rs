mod graphics;

fn main() {
    use graphics::GraphicsContext;

    stderrlog::new().verbosity(4).init().unwrap();

    let GraphicsContext { event_loop, window: _window, .. } =
        graphics::setup_graphics();

    // FIXME:
    //   If the window variable goes out of scope,
    //   there is no window to receive a close request
    //   and this loop never terminates.
    //   Make sure that this loop always terminates
    //   if the window ceases to exist.
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
