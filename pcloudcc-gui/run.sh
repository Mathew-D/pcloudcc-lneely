#!/bin/fish
export WINIT_UNIX_BACKEND=x11
export SDL_VIDEODRIVER=x11
./target/debug/pcloudcc-gui "$@"
