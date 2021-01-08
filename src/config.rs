use std::boxed::Box;
use std::collections::HashMap;
use std::default::Default;
use std::str::FromStr;
use uuid::Uuid;

use strict_yaml_rust::strict_yaml::{StrictYaml, StrictYamlLoader, Hash};
use winit::dpi::PhysicalSize;

use crate::Config;
use crate::audio::{
    AudioConfig,
    AudioHostConfig,
};
use crate::graphics::{
    DeviceSelection,
    GraphicsConfig,
    InstanceConfig,
    LayersConfig,
    MonitorConfig,
    VideoModeConfig,
    VulkanConfig,
    WindowedConfig,
    WindowConfig,
    WindowMode,
};

fn read_config_file() -> std::io::Result<Box<str>> {
    std::fs::read_to_string(std::path::Path::new("config/config.yml"))
        .map(String::into_boxed_str)
}

fn yaml_get<'a>(cfg: &'a Hash, name: &str) -> Option<&'a StrictYaml> {
    cfg.get(&StrictYaml::String(name.to_string()))
}

fn parse_section<F, U>(cfg: &Hash, name: &str, parse: F) -> U
where
    F: FnOnce(&StrictYaml) -> U,
    U: Default
{
    yaml_get(cfg, name)
        .map_or_else(
            || {
                log::warn!(
                    "Config file is missing the `{}` section! \
                     Using default value.",
                    name
                );
                U::default()
            },
            parse
        )
}

fn parse_auto_str<'a>(cfg: &'a Hash, name: &str) -> Option<&'a str> {
    yaml_get(cfg, name)
        .map_or_else(
            || {
                log::warn!(
                    "Config file is missing the `{}` option! \
                     Using default value.",
                    name
                );
                None
            },
            |node| {
                let str = node.as_str().expect("Auto string was not a string");
                if str == "auto" {
                    None
                } else {
                    Some(str)
                }
            }
        )
}

fn parse_auto_boxed_str<'a>(cfg: &Hash, name: &str) -> Option<Box<str>> {
    parse_auto_str(cfg, name).map(|x| x.to_string().into_boxed_str())
}

fn parse_from_str<T, E>(cfg: &Hash, name: &str, default: T) -> T
where
    T: FromStr<Err=E>,
    E: std::fmt::Debug,
{
    yaml_get(cfg, name)
        .map_or_else(
            || {
                log::warn!(
                    "Config file is missing the `{}` option! \
                     Using default value.",
                    name
                );
                default
            },
            |node| {
                let st = node.as_str().expect("Option was not a string");
                T::from_str(st).expect("Option was of incorrect type")
            }
        )
}

fn parse_from_str_option<T, E>(cfg: &Hash, name: &str) -> Option<T>
where
    T: FromStr<Err=E>,
    E: std::fmt::Debug,
{
    yaml_get(cfg, name)
        .map_or_else(
            || {
                log::warn!(
                    "Config file is missing the `{}` option! \
                     Using default value.",
                    name
                );
                None
            },
            |node| {
                let st = node.as_str().expect("Option was not a string");
                if st == "auto" {
                    None
                } else {
                    Some(T::from_str(st).expect("Option was of incorrect type"))
                }
            }
        )
}

fn parse_host(host_node: &StrictYaml) -> AudioHostConfig {
    let host = host_node.as_hash().expect("Host section was not a hash");
    let output_device = parse_auto_boxed_str(host, "output_device");
    AudioHostConfig { output_device }
}

fn parse_hosts(hosts_node: &StrictYaml) -> HashMap<Box<str>, AudioHostConfig> {
    let hosts =
        hosts_node.as_hash().expect("Hosts section was not a hash");
    let mut out = HashMap::new();
    for (name, host_node) in hosts {
        out.insert(
            name.as_str().unwrap().into(),
            parse_host(host_node)
        );
    }
    out
}

fn parse_audio_config(audio_node: &StrictYaml) -> AudioConfig {
    let audio =
        audio_node.as_hash().expect("Audio section was not a hash");
    let host = parse_auto_boxed_str(audio, "host");
    let hosts = parse_section(audio, "hosts", parse_hosts);
    AudioConfig { host, hosts }
}

fn parse_device_selection(device_node: &StrictYaml) -> DeviceSelection {
    let device =
        device_node.as_str().expect("Device selection was not a string");
    match device {
        "auto" => DeviceSelection::Auto,
        "best" => DeviceSelection::Best,
        _ => DeviceSelection::Uuid(
            Uuid::parse_str(device).expect("Device selection was not a UUID")
        )
    }
}

fn parse_layer_option(layers: &Hash, name: &str) -> bool {
    // TODO: Support `auto` option as well.
    yaml_get(layers, name)
        .map_or_else(
            || {
                log::warn!(
                    "Config file is missing the `{}` option! \
                     Using default value.",
                    name
                );
                false
            },
            |node| {
                bool::from_str(
                    node.as_str().expect("Layer option was not a bool"))
                    .expect("Layer option was not a bool")
            }
        )
}

fn parse_layers_config(layers_node: &StrictYaml) -> LayersConfig {
    let layers =
        layers_node.as_hash().expect("Layers section was not a hash");
    let mesa_device_select = parse_layer_option(layers, "mesa_device_select");
    let mesa_overlay = parse_layer_option(layers, "mesa_overlay");
    let khronos_validation = parse_layer_option(layers, "khronos_validation");
    LayersConfig { mesa_device_select, mesa_overlay, khronos_validation }
}

fn parse_instance_config(instance_node: &StrictYaml) -> InstanceConfig {
    let instance =
        instance_node.as_hash().expect("Instance section was not a hash");
    let layers = parse_section(instance, "layers", parse_layers_config);
    InstanceConfig { layers }
}

fn parse_window_mode(window: &Hash) -> WindowMode {
    yaml_get(window, "mode").
        map_or_else(
            || {
                log::warn!(
                    "Config file is missing the `mode` option! \
                     Using default value."
                );
                WindowMode::Windowed
            },
            |node| {
                match node.as_str().expect("Mode option was not a string") {
                    "borderless" => WindowMode::BorderlessFullscreen,
                    "exclusive" => WindowMode::ExclusiveFullscreen,
                    "windowed" => WindowMode::Windowed,
                    mode => panic!("Unknown window mode: {}", mode)
                }
            }
        )
}

fn parse_video_mode_config(
    video_mode_node: &StrictYaml
) -> Option<VideoModeConfig> {
    let video_mode =
        video_mode_node.as_hash().expect("Video mode section was not a hash");
    let width = parse_from_str_option(video_mode, "width").unwrap();
    let height = parse_from_str_option(video_mode, "height").unwrap();
    let bit_depth = parse_from_str_option(video_mode, "bit_depth").unwrap();
    let refresh_rate =
        parse_from_str_option(video_mode, "refresh_rate").unwrap();
    Some(VideoModeConfig {
        size: PhysicalSize::new(width, height),
        bit_depth,
        refresh_rate,
    })
}

fn parse_monitor_config(monitor_node: &StrictYaml) -> MonitorConfig {
    let monitor =
        monitor_node.as_hash().expect("Monitor section was not a hash");
    let scale_factor = parse_from_str_option(monitor, "scale_factor");
    let video_mode =
        parse_section(monitor, "video_mode", parse_video_mode_config)
        .expect("There is no default value for video modes");
    MonitorConfig { scale_factor, video_mode }
}

fn parse_monitors(
    monitors_node: &StrictYaml
) -> HashMap<Box<str>, MonitorConfig> {
    let monitors =
        monitors_node.as_hash().expect("Monitors section was not a hash");
    let mut out = HashMap::new();
    for (name, monitor_node) in monitors {
        out.insert(
            name.as_str().unwrap().into(),
            parse_monitor_config(monitor_node)
        );
    }
    out
}

fn parse_windowed_config(windowed_node: &StrictYaml) -> WindowedConfig {
    let windowed =
        windowed_node.as_hash().expect("Windowed section was not a hash");
    let width = parse_from_str(windowed, "width", 1280);
    let height = parse_from_str(windowed, "height", 720);
    let always_on_top = parse_from_str(windowed, "always_on_top", false);
    let decorations = parse_from_str(windowed, "decorations", false);
    let maximized = parse_from_str(windowed, "maximized", true);
    let resizable = parse_from_str(windowed, "resizable", true);
    let scale_factor = parse_from_str_option(windowed, "scale_factor");
    WindowedConfig {
        size: PhysicalSize::new(width, height),
        always_on_top,
        decorations,
        maximized,
        resizable,
        scale_factor,
    }
}

fn parse_window_config(window_node: &StrictYaml) -> WindowConfig {
    let window =
        window_node.as_hash().expect("Window section was not a hash");
    let mode = parse_window_mode(window);
    let monitor = parse_auto_boxed_str(window, "monitor");
    let monitors = parse_section(window, "monitors", parse_monitors);
    let windowed = parse_section(window, "windowed", parse_windowed_config);
    WindowConfig { mode, monitor, monitors, windowed }
}

fn parse_graphics_config(graphics_node: &StrictYaml) -> GraphicsConfig {
    let graphics =
        graphics_node.as_hash().expect("Graphics section was not a hash");
    let device = parse_section(graphics, "device", parse_device_selection);
    let instance = parse_section(graphics, "instance", parse_instance_config);
    let window = parse_section(graphics, "window", parse_window_config);
    GraphicsConfig { vulkan: VulkanConfig { device, instance }, window }
}

fn parse_config(root_node: &StrictYaml) -> Config {
    let root = root_node.as_hash().expect("Config root was not a hash.");
    let audio = parse_section(root, "audio", parse_audio_config);
    let graphics = parse_section(root, "graphics", parse_graphics_config);
    Config { audio, graphics }
}

pub fn read_config() -> Config {
    read_config_file().map_or_else(
        |_| {
            log::warn!("Unable to open config file! Using default config.");
            Config::default()
        }, |file_contents| {
            let data = StrictYamlLoader::load_from_str(&file_contents).unwrap();
            parse_config(data.first().unwrap())
        })
}
