#!/bin/bash
set -e

# Note, I had to update the bindgen dependency of ffmpeg-sys-next so the versions of clang wouldn't conflict with opencv

# sudo apt-get install -y libsdl2-dev
sudo apt install -y \
  clang libavcodec-dev libavformat-dev libavutil-dev libavfilter-dev libavdevice-dev pkg-config yasm \
  libopencv-dev llvm libclang-10-dev clang # for opencv
cargo build
