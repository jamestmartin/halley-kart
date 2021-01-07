use std::boxed::Box;
use std::collections::HashMap;
use std::default::Default;

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::monitor::{MonitorHandle, VideoMode};
use winit::window::{Fullscreen, Window};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WindowMode {
    Windowed,
    ExclusiveFullscreen,
    BorderlessFullscreen,
}

impl Default for WindowMode {
    fn default() -> Self {
        WindowMode::Windowed
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowedConfig {
    pub size: PhysicalSize<u32>,
    pub always_on_top: bool,
    pub decorations: bool,
    pub maximized: bool,
    pub resizable: bool,
    pub scale_factor: Option<f64>,
}

impl Default for WindowedConfig {
    fn default() -> Self {
        WindowedConfig {
            size: PhysicalSize::new(1280, 720),
            always_on_top: false,
            decorations: false,
            maximized: true,
            resizable: true,
            scale_factor: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoModeConfig {
    pub size: PhysicalSize<u32>,
    pub bit_depth: u16,
    pub refresh_rate: u16,
}

impl VideoModeConfig {
    fn matches(&self, mode: &VideoMode) -> bool {
        self.size == mode.size()
            && self.bit_depth == mode.bit_depth()
            && self.refresh_rate == mode.refresh_rate()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MonitorConfig {
    pub scale_factor: Option<f64>,
    pub video_mode: VideoModeConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowConfig {
    pub mode: WindowMode,
    pub windowed: WindowedConfig,
    pub monitor: Option<Box<str>>,
    pub monitors: HashMap<Box<str>, MonitorConfig>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            mode: WindowMode::default(),
            windowed: WindowedConfig::default(),
            monitor: None,
            monitors: HashMap::new(),
        }
    }
}

fn primary_monitor<T>(
    window_target: &EventLoopWindowTarget<T>
) -> MonitorHandle {
    window_target.primary_monitor().or_else(
        || window_target.available_monitors().next())
        .expect("Could not find any monitor for fullscreen!")
}

fn find_monitor<T>(
    name: &str,
    window_target: &EventLoopWindowTarget<T>
) -> Option<MonitorHandle> {
    window_target.available_monitors().find(|m| name == m.name().unwrap())
}

fn find_best_video_mode(monitor: MonitorHandle) -> VideoMode {
    // TODO: Choose the actual best video mode, not just the first.
    monitor.video_modes().next().unwrap()
}

fn find_video_mode(
    monitor: MonitorHandle,
    config: Option<VideoModeConfig>
) -> VideoMode {
    config.and_then(|cfg| monitor.video_modes().find(|m| cfg.matches(m)))
          .unwrap_or_else(|| find_best_video_mode(monitor))
}

fn fullscreen_mode<T>(
    window_target: &EventLoopWindowTarget<T>,
    config: &WindowConfig
) -> Option<Fullscreen> {
    if config.mode == WindowMode::Windowed {
        return None;
    }

    let opt_monitor =
        config.monitor.as_ref().and_then(
            |name| find_monitor(&name, window_target));

    if opt_monitor.is_none() && config.monitor.is_some() {
        log::warn!("Could not find requested monitor: {}",
                   config.monitor.as_ref().unwrap());
    }

    if config.mode == WindowMode::BorderlessFullscreen {
        return Some(Fullscreen::Borderless(opt_monitor));
    }

    let monitor: MonitorHandle =
        opt_monitor.unwrap_or_else(|| primary_monitor(window_target));
    let video_mode_config =
        config.monitors.get(&monitor.name().unwrap().into_boxed_str())
                       .map(|c| c.video_mode.clone());

    Some(Fullscreen::Exclusive(
        find_video_mode(monitor, video_mode_config)))
}

pub fn build_window<T: 'static>(
    window_target: &EventLoopWindowTarget<T>,
    config: &WindowConfig
) -> Result<Window, winit::error::OsError> {
    let builder = winit::window::WindowBuilder::new()
        .with_title("Halley Kart");
    if config.mode == WindowMode::Windowed {
        builder
            .with_visible(false)
            .with_resizable(config.windowed.resizable)
            .with_inner_size(config.windowed.size)
            .with_maximized(config.windowed.maximized)
            .with_always_on_top(config.windowed.always_on_top)
            .with_decorations(config.windowed.decorations)
    } else {
        builder
            .with_fullscreen(fullscreen_mode(window_target, config))
    }.build(window_target)
}
