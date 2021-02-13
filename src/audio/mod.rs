use std::boxed::Box;
use std::collections::HashMap;
use std::default::Default;

use cpal::{Device, Host, Stream, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AudioConfig {
    pub host: Option<Box<str>>,
    pub hosts: HashMap<Box<str>, AudioHostConfig>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AudioHostConfig {
    pub output_device: Option<Box<str>>,
}

pub struct AudioContext {
    host: Host,
    device: Device,
    config: SupportedStreamConfig,
    stream: Stream,
}

fn select_host(host_nameq: Option<&str>) -> Host {
    if let Some(host_name) = host_nameq {
        let hosts = cpal::ALL_HOSTS;
        let host_idq = hosts.iter().find(|id| id.name() == host_name);
        if let Some(host_id) = host_idq {
            if let Ok(host) = cpal::host_from_id(*host_id) {
                return host;
            } else {
                log::warn!(
                    "Requested audio host `{}` was not available, \
                     falling back to platform-default host.",
                    host_name
                );
            }
        } else {
            log::warn!(
                "Requested audio host `{}`  does not exist on this platform! \
                 Was the game compiled with support for it? \
                 Falling back to platform-default host.",
                host_name
            );
        }
    }

    cpal::default_host()
}

fn select_output_device<D>(
    host: &impl HostTrait<Device=D>,
    device_nameq: Option<&str>
) -> Option<D>
where
    D: DeviceTrait,
{
    if let Some(device_name) = device_nameq {
        match host.output_devices() {
            Err(err) => {
                log::warn!(
                    "Failed to enumerate output devices! \
                     Falling back to default audio device. Error: {}",
                    err
                );
            }, Ok(mut output_devices) => {
                let output_deviceq = output_devices.find(|device| {
                    device.name().map_or_else(
                        |err| {
                            log::warn!(
                                "Failed to get output device name: {}",
                                err
                            );
                            false
                        }, |name| name == device_name)
                });
                if let Some(output_device) = output_deviceq {
                    return Some(output_device);
                } else {
                    log::warn!(
                        "Requested audio output device `{}` was not available, \
                         falling back to default device.",
                        device_name
                    );
                }
            }
        }
    }

    let default_device = host.default_output_device();
    if default_device.is_none() {
        log::error!("No audio output devices are available! \
                     Game audio will be disabled.");
    }
    default_device
}

fn select_output_config(
    device: &impl DeviceTrait
) -> Option<SupportedStreamConfig> {
    device.default_output_config().map_err(
        |err| log::error!(
            "Error retrieving default output config for device: {}",
            err
        )
    ).ok()
}

fn data_callback(data: &mut [f32], _info: &cpal::OutputCallbackInfo) {
    //use cpal::Sample;
    //let mut i = 0;
    for sample in data {
        //*sample = Sample::from(&(u16::MAX / 2 / (i + 1)));
        //i += 1;
        *sample = 0.0;
    }
}

fn error_callback(err: cpal::StreamError) {
    log::warn!("Stream error: {}", err);
}

impl AudioContext {
    pub fn create(config: &AudioConfig) -> Option<Self> {
        let host = select_host(config.host.as_deref());
        log::info!("Using audio host: {}", host.id().name());
        let host_config = config.hosts.get(host.id().name());
        let device_name = host_config
            .and_then(|hc| hc.output_device.as_deref());
        let device = select_output_device(&host, device_name)?;
        log::info!("Using audio output device: {}", device.name().unwrap());
        let config = select_output_config(&device)?;

        let config2 = cpal::StreamConfig {
            //buffer_size: cpal::BufferSize::Fixed(config.sample_format().sample_size() as u32 * 2 * 1024),
            .. config.config()
        };
        log::trace!("Config: {:?}", config.config());
        let stream = device.build_output_stream(
            &config2,
            data_callback,
            error_callback
        ).expect("Failed to create audio output stream");

        Some(Self { host, device, config, stream })
    }
}
