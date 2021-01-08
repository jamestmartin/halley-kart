# Halley Kart
A racing game. This project is experimental, please disregard for now.

## Requirements
As a rule of thumb:
You shouldn't have to install anything for Halley Kart to work,
and most hardware released after 2014 (and some before, because of Vulkan) should support it.

* CPU: x86-64
* OS: Linux, Windows
* Graphics: Vulkan 1.0 (most GPUs released ~2014+ and some earlier should support it)
* Audio:
    * Linux: ALSA (yes, you support it)
    * Windows: WASAPI (yes, you support it)
* Controllers:
    * Linux: Linux Gamepad API, any controller with a sane layout should work
        * Tested: Xbox 360 controller
        * If your controller works and isn't listed here, please let me know.
    * Windows: Xinput
        * Tested: Windows hasn't been tested thoroughly yet!
        * If your controller works and isn't listed here, please let me know.
* Windowing Systems: X11, Wayland, Windows

Some other platforms may work but are not tested or officially supported.

I can only support the hardware that I actually personally own.
If you'd like to help develop and/or test for other platforms (CPUs, OSes, or even consoles),
I would love your help! Please let me know if you're interested.

## Installation
Right now, the only way to install this game is to build from source, see below.

## Building
### Linux
On Debian-based distributions (e.g. Ubuntu), install:
* `libasound2-dev` (required by `cpal` to interface with ALSA)
* `libjack-jackd2-dev` (required by `cpal` to interface with JACK
  if the `audio-backend-jack` feature is enabled)
* `libudev-dev` (required by `gilrs` to access gamepads)

If you do not already have the `shaderc` library installed on your system
(there is currently no package to use to install it in Debian),
you will need to additionally install these tools
so that the `shaderc-sys` crate can build it from scratch for you:
* CMake
* Git
* Python 3
* a C++11 compiler

On other distributions, you will need to figure out how to install the equivalent headers.
(If you want to make a PR to list the equivalent packages on your distro, feel free.)

Then simply build with `cargo build` and run with `cargo run`, as usual.

### Windows
If you do not already have the `shaderc` library installed on your system,
you will need to additionally install these tools and add them to your PATH
so that the `shaderc-sys` crate can build it from scratch for you:
* CMake
* Git
* Python 3
* a C++11 compiler
* Ninja (only if your platform is `windows-msvc`)

Then simply build with `cargo build` and run with `cargo run`, as usual.
