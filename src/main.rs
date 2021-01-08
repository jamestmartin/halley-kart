mod audio;
mod config;
mod graphics;

use audio::AudioConfig;
use graphics::GraphicsConfig;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub audio: AudioConfig,
    pub graphics: GraphicsConfig,
}

fn main() {
    use graphics::GraphicsContext;

    stderrlog::new().verbosity(3).init().unwrap();

    let config = config::read_config();

    let _audio_context = audio::setup_audio(&config.audio);

    let GraphicsContext { event_loop, window: _window, .. } =
        graphics::setup_graphics(&config.graphics);

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
                log::trace!("Window close requested.");
            },
            LoopDestroyed => {
                log::info!("Goodbye!");
            },
            _ => ()
        }
    });
}
