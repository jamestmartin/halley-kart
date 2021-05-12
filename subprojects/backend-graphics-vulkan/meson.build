project('hk-backend-graphics-vulkan', 'c')

vulkan = dependency('vulkan')

library(
    'hk-backend-graphics-vulkan',
    'src/vulkan.c',
    dependencies: vulkan,
    name_prefix: '../../',
    install: true,
    install_dir: 'lib/x86_64-linux-gnu/halley-kart',
)
