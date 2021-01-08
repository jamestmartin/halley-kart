use std::thread;
use uuid::Uuid;

use gilrs::Gilrs;

fn main() {
    thread::spawn(dump_gamepad_stuff);
    dump_window_stuff();
}

fn dump_gamepad_stuff() {
    let mut gilrs = Gilrs::new().unwrap();
    dump_connected_gamepads(&gilrs);
    dump_gamepad_inputs(&mut gilrs)
}

fn dump_connected_gamepads(gilrs: &Gilrs) {
    println!("Connected gamepads:");
    for (_gamepad_id, gamepad) in gilrs.gamepads() {
        let uuid = Uuid::from_bytes(gamepad.uuid());
        println!("* {} (UUID: {}", gamepad.name(), uuid);
        println!("  * OS name: {}", gamepad.os_name());
        println!("  * Map name: {:?}", gamepad.map_name());
        println!("  * Mapping source {:?}", gamepad.mapping_source());
        println!("  * Force feedback supported: {}", gamepad.is_ff_supported());
        println!("  * Power info: {:?}", gamepad.power_info());
        // TODO: Print deadzones.
        // TODO: Print each available axis and button?
    }
}

fn dump_gamepad_inputs(gilrs: &mut Gilrs) {
    loop {
        while let Some(event) = gilrs.next_event() {
            println!("Gamepad event: {:?}", event);
        }

        thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn dump_window_stuff() {
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;

    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("HK Dump Input")
        .build(&event_loop)
        .unwrap();

    event_loop.run(|event, _, control_flow| {
        use winit::event::{DeviceEvent, Event, WindowEvent};
        use winit::event_loop::ControlFlow;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
                println!("Window close requested");
            },
            Event::LoopDestroyed => {
                println!("Window loop destroyed. Goodbye!");
            },
            // Events that will spam the log without providing
            // much useful information.
            Event::MainEventsCleared => {},
            Event::RedrawRequested(_) => {},
            Event::RedrawEventsCleared => {},
            Event::NewEvents(_) => {},
            // Events that do provide some useful information,
            // but are too spammy nonetheless.
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { .. }, .. } => {},
            Event::DeviceEvent { event: DeviceEvent::Motion { axis: 0, .. }, .. } => {},
            Event::DeviceEvent { event: DeviceEvent::Motion { axis: 1, .. }, .. } => {},
            _ => {
                println!("Window event: {:?}", event);
            }
        }
    });
}
