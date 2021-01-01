#!/bin/bash
set -e

# sudo apt-get install -y libsdl2-dev
sudo apt install -y clang libavcodec-dev libavformat-dev libavutil-dev libavfilter-dev libavdevice-dev pkg-config \
  libopencv-dev llvm libclang-10-dev clang # for opencv
cargo build
