project(
    'halley-kart',
    'c',
    default_options: ['c_std=c17', 'warning_level=3']
)

add_project_arguments('-fPIE', '-pie', '-Bsymbolic', '-shared', language: 'c')
add_project_link_arguments('-rdynamic', language: 'c')

dl = meson.get_compiler('c').find_library('dl', required: false)

if not dl.found()
    warning(
      '''
      DL library not found; the generated executable
      may not work on some glibc-based systems
      '''
    )
endif

executable(
    'halley-kart',
    ['src/main.c', 'src/plugin-manager.c'],
    include_directories: 'include',
    dependencies: dl,
    install: true,
)

subproject('make-plugin-elf-notes')

alsa = dependency('alsa', required: false)
vulkan = dependency('vulkan', required: false)
wayland_client = dependency('wayland-client', required: false)

if alsa.found()
    subproject('backend-audio-alsa')
else
    warning('ALSA library not found; skipping ALSA audio backend')
endif

if vulkan.found()
    subproject('backend-graphics-vulkan')
else
    warning('Vulkan library not found; skipping Vulkan graphics backend')
endif

if wayland_client.found()
    subproject('backend-display-wayland')
else
    warning('Wayland library not found; skipping Wayland display backend')
endif
