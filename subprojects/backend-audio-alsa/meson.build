project('hk-backend-audio-alsa', 'c')

add_project_link_arguments(
    '-Wl,--allow-shlib-undefined',
    '-Wl,--unresolved-symbols=ignore-all',
    language: 'c'
)

make_plugin_elf_notes_proj = subproject('make-plugin-elf-notes')

make_plugin_elf_notes = make_plugin_elf_notes_proj.get_variable('make_elf_notes')
plugin_elf_notes = custom_target(
    'make_elf_notes',
    output: 'notes.o',
    command: [
      make_plugin_elf_notes,
      '--optional-dynamic',
      meson.current_build_dir() / 'notes.o',
      'hkal_get_procs',
    ],
)

alsa = dependency('alsa')

library(
    'hk-backend-audio-alsa',
    ['src/alsa.c', plugin_elf_notes],
    include_directories: '../../include',
    dependencies: alsa,
    name_prefix: '../../plugins/',
    install: true,
    install_dir: 'lib/x86_64-linux-gnu/halley-kart',
)
