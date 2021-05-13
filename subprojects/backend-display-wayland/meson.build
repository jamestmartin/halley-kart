project('hk-backend-graphics-vulkan', 'c')

add_project_link_arguments(
    '-Wl,--allow-shlib-undefined',
    '-Wl,--unresolved-symbols=ignore-all',
    language: 'c'
)

make_plugin_elf_notes_proj = subproject('make-plugin-elf-notes')
wayland_protocols_proj = subproject('wayland-protocols')

make_plugin_elf_notes = make_plugin_elf_notes_proj.get_variable('make_elf_notes')
plugin_elf_notes = custom_target(
    'make_elf_notes',
    output: 'notes.o',
    command: [
      make_plugin_elf_notes,
      '--optional-dynamic',
      meson.current_build_dir() / 'notes.o',
      'hkwl_get_procs',
    ],
)

rt = meson.get_compiler('c').find_library('rt')
wayland_client = dependency('wayland-client')
wayland_protocols = wayland_protocols_proj.get_variable(
        'libwayland_protocols_dep'
    )

library(
    'hk-backend-display-wayland',
    ['src/wayland.c', plugin_elf_notes],
    include_directories: '../../include',
    dependencies: [rt, wayland_client, wayland_protocols],
    name_prefix: '../../plugins/',
    install: true,
    install_dir: 'lib/x86_64-linux-gnu/halley-kart',
)
