use std::sync::Arc;
use uuid::Uuid;

use cpal::traits::{HostTrait, DeviceTrait};
use gilrs::Gilrs;
use gumdrop::Options;
use vulkano::instance::{
    Instance,
    InstanceExtensions,
    PhysicalDevice,
    RawInstanceExtensions
};

#[derive(Debug, Options)]
struct DumpOptions {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "include a *lot* of detail")]
    verbose: bool,
}

fn main() {
    let opts = DumpOptions::parse_args_default_or_exit();
    let verbose = opts.verbose;

    dump_audio_options(verbose);
    dump_vulkan_options(verbose);
    dump_gamepad_options();
}

fn dump_audio_options(verbose: bool) {
    println!("Audio hosts: ");
    for host_id in cpal::ALL_HOSTS {
        print!("* {}", host_id.name());
        let host =  match cpal::host_from_id(*host_id) {
            Err(_) => {
                println!(" (unavailable)");
                continue
            },
            Ok(host) => host
        };
        println!();
        let output_devicesq = host.output_devices();
        match output_devicesq {
            Err(err) => {
                println!("  * Error: {}", err);
            },
            Ok(output_devices) => {
                for device in output_devices {
                    match device.name() {
                        Ok(name) => {
                            println!("    * {}", name);
                        },
                        Err(err) => {
                            println!("    * Error: {}", err);
                        }
                    }
                    match device.default_output_config() {
                        Ok(cfg) => {
                            println!("      * Default: {:?}", cfg);
                        },
                        Err(err) => {
                            println!("      * Default Error: {}", err);
                        }
                    }
                    if !verbose { continue }
                    match device.supported_output_configs() {
                        Ok(cfgs) => {
                            for cfg in cfgs {
                                println!("      * {:?}", cfg);
                            }
                        },
                        Err(err) => {
                            println!("      * {}", err);
                        }
                    }
                }
            }
        }
    }
}

fn dump_vulkan_options(verbose: bool) {
    dump_vulkan_instance_layers();
    dump_vulkan_instance_extensions();

    let instance_extensions = InstanceExtensions::none();
    let instance = match Instance::new(None, &instance_extensions, vec![]) {
        Ok(x) => x,
        Err(err) => {
            println!("Failed to create Vulkan instance: {}", err);
            return
        }
    };

    dump_vulkan_physical_devices(&instance, verbose);
}

fn dump_vulkan_instance_layers() {
    use vulkano::instance::layers_list;

    println!("Vulkan instance layers:");
    match layers_list() {
        Ok(layers) => {
            for layer in layers {
                println!("* {}", layer.name());
            }
        }, Err(err) => {
            println!("* Error: {}", err);
        }
    }
}

fn dump_vulkan_instance_extensions() {
    println!("Vulkan instance extensions:");
    match RawInstanceExtensions::supported_by_core_raw() {
        Ok(extensions) => {
            for extension in extensions.iter() {
                let name = extension.to_str().expect("Bad extension name");
                println!("* {}", name);
            }
        }, Err(err) => {
            println!("* Error: {}", err);
        }
    }
}

fn dump_vulkan_physical_devices(instance: &Arc<Instance>, verbose: bool) {
    println!("Vulkan physical devices:");
    for physical_device in PhysicalDevice::enumerate(instance) {
        dump_vulkan_physical_device(&physical_device, verbose);
    }
}


fn dump_vulkan_physical_device(
    physical_device: &PhysicalDevice<'_>,
    verbose: bool
) {
    let uuid = Uuid::from_bytes(*physical_device.uuid());

    println!("* {} (UUID: {})", physical_device.name(), uuid);
    println!("  * Type: {:?}", physical_device.ty());
    println!("  * API version: {}", physical_device.api_version());
    println!("  * Driver version: {}", physical_device.driver_version());
    println!("  * PCI device ID: {}", physical_device.pci_device_id());
    println!("  * PCI vendor ID: {}", physical_device.pci_vendor_id());
    if verbose {
        println!(
            "  * Supported features: {:?}",
            physical_device.supported_features()
        );
    }

    println!("  * Queue families:");
    for queue_family in physical_device.queue_families() {
        println!(
            "    * Id: {}, Count: {}, Graphics: {}, Compute: {}, Transfers: {}",
            queue_family.id(),
            queue_family.queues_count(),
            queue_family.supports_graphics(),
            queue_family.supports_compute(),
            queue_family.explicitly_supports_transfers()
        );
    }
}

fn dump_gamepad_options() {
    let gilrs = Gilrs::new().unwrap();

    println!("Connected gamepads:");
    for (_gamepad_id, gamepad) in gilrs.gamepads() {
        let uuid = Uuid::from_bytes(gamepad.uuid());
        println!("* {} (UUID: {}", gamepad.name(), uuid);
        println!("  * OS name: {}", gamepad.os_name());
        println!("  * Map name: {:?}", gamepad.map_name());
        println!("  * Mapping source {:?}", gamepad.mapping_source());
        println!("  * Force feedback supported: {}", gamepad.is_ff_supported());
        println!("  * Power info: {:?}", gamepad.power_info());
    }
}
