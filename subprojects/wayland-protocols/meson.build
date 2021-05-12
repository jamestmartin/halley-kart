project('wayland-protocols', 'c')

rt = meson.get_compiler('c').find_library('rt')
wayland_client = dependency('wayland-client')

build_dir = meson.current_build_dir()
gen_protocol = generator(
    find_program(join_paths(meson.current_source_dir(), 'scan-protocol.sh')),
    output: ['@BASENAME@-protocol.c', '@BASENAME@-client-protocol.h'],
    arguments: [meson.current_source_dir(), build_dir, '@BUILD_DIR@', '@BASENAME@']
)

xdg_shell_protocol = gen_protocol.process('protocols/xdg-shell')

# Meson artifically restricts you from having absolute paths,
# even though morally this is a relative path anyway...
#inc = include_directories(join_paths(meson.current_build_dir(), 'include'))
thanks_meson = run_command(
    join_paths(meson.current_source_dir(), 'fix-path.sh'),
    meson.current_build_dir(),
)
inc = include_directories(thanks_meson.stdout())

libwayland_protocols = static_library(
    'wayland-protocols',
    xdg_shell_protocol,
    include_directories: inc,
    dependencies: [
      rt,
      wayland_client,
    ],
)

libwayland_protocols_dep = declare_dependency(
    include_directories: inc,
    link_with: libwayland_protocols,
)
