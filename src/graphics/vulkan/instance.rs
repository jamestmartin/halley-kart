use std::boxed::Box;
use std::sync::Arc;

use vulkano::instance::Instance;

/// The purpose of the ApplicationInfo struct is to identify the game and engine
/// to the Vulkan driver so that it can apply optimizations
/// specialized to particular game engines.
///
/// This game does not use any pre-existing engine,
/// instead opting for a custom, game-specific engine,
/// and thus is unlikely to ever be recognized by any driver.
/// Nonetheless, the fields are populated with this game's information.
fn application_info<'a>() -> vulkano::instance::ApplicationInfo<'a> {
    use vulkano::instance::ApplicationInfo;
    use vulkano::instance::Version;

    // If you fork this game, you should change the application name,
    // but not the engine name. Perhaps one day a driver will
    // recognize the game, and optimizations for our engine
    // would then be able to benefit your fork as well.
    //
    // Also, if it ever gets to that point, performance-wise,
    // it may be worth spoofing the engine to pretend to be
    // another engine with similar performance characteristics
    // (only after extensive testing, of course).
    //
    // TODO: Automatically retrieve this data from the Cargo.toml or similar.
    ApplicationInfo {
        application_name: Some("Halley Kart".into()),
        application_version: Some(Version { major: 0, minor: 1, patch: 0 }),
        engine_name: Some("Halley Kart".into()),
        engine_version: Some(Version { major: 0, minor: 1, patch: 0 }),
    }
}

/// These are the instance extensions supported by the Vulkan implementation
/// that we have detected and are able to take advantage of.
///
/// This function will panic if the Vulkan implementation
/// lacks the extensions required for the game to function,
/// seeing as there is little we can meaningfully do to recover
/// in that scenario.
fn instance_extensions() -> vulkano::instance::InstanceExtensions {
    use vulkano::instance::InstanceExtensions;

    let available = InstanceExtensions::supported_by_core()
        .expect("Failed to enumerate supported Vulkan instance extensions.");
    let required = vulkano_win::required_extensions();

    let missing = required.difference(&available);
    if missing != InstanceExtensions::none() {
        panic!("Missing required Vulkan instance extensions: {:?}", missing);
    }

    let extensions = available.intersection(&required);
    log::debug!("Using Vulkan instance extensions: {:?}", extensions);

    extensions
}

/// Detect what instance layers are supported by the Vulkan implementations
/// and select the ones that we'd like to use.
///
/// This function will panic if the supported layers can't be listed,
/// because even though we function just fine without any layers enabled,
fn instance_layers() -> Box<[Box<str>]> {
    use vulkano::instance::layers_list;

    // TODO: Support VK_LAYER_MESA_overlay for debugging purposes.
    //
    // TODO:
    //   Investigate VK_LAYER_MESA_device_select. I can't find documentation on it,
    //   so it's unclear what it actually does, but I suspect that it
    //   could be used to automatically pick a good device using better logic
    //   than I'd be likely to implement.
    //   The DRI_PRIME environment variable appears to change the order
    //   in which physical devices appear; perhaps the layer works similarly?
    //
    // TODO:
    //   Test whether VK_LAYER_KHRONOS_validation has any measurable
    //   performance impact, and if it does, provide a means to disable it.
    //   I don't expect that it will, but we might as well wait and see.
    let desired = ["VK_LAYER_KHRONOS_validation"];

    let mut layers = Vec::new();
    let mut ignored_layers = Vec::new();
    for layer in layers_list().expect("Unable to list Vulkan instance layers.") {
        let name = layer.name();
        if desired.contains(&name) {
            layers.push(String::from(name).into_boxed_str());
        } else {
            ignored_layers.push(String::from(name).into_boxed_str());
        }
    }

    log::debug!("Using Vulkan instance layers: {:?}", layers);
    log::debug!("Ignoring Vulkan instance layers: {:?}", ignored_layers);

    layers.into_boxed_slice()
}

/// Create a Vulkan instance with the necessary extensions and layers.
///
/// This function will panic if we fail to create the Vulkan instance,
/// seeing as there's no reasonable way to recover from that.
pub fn create_instance() -> Arc<Instance> {
    let exts = vulkano::instance::RawInstanceExtensions::from(&instance_extensions());
    Instance::new(Some(&application_info()),
                  exts,
                  instance_layers().into_iter().map(AsRef::as_ref))
        .expect("Failed to create Vulkan instance.")
}
