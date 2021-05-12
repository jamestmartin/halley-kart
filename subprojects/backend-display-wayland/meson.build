project('hk-backend-graphics-vulkan', 'c')

wayland_protocols_proj = subproject('wayland-protocols')

rt = meson.get_compiler('c').find_library('rt')
wayland_client = dependency('wayland-client')
wayland_protocols = wayland_protocols_proj.get_variable(
        'libwayland_protocols_dep'
    )

library(
    'hk-backend-display-wayland',
    'src/wayland.c',
    dependencies: [rt, wayland_client, wayland_protocols],
    name_prefix: '../../',
    install: true,
    install_dir: 'lib/x86_64-linux-gnu/halley-kart',
)
