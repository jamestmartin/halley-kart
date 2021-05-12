project('hk-backend-audio-alsa', 'c')

alsa = dependency('alsa')

library(
    'hk-backend-audio-alsa',
    'src/alsa.c',
    dependencies: alsa,
    name_prefix: '../../',
    install: true,
    install_dir: 'lib/x86_64-linux-gnu/halley-kart',
)
