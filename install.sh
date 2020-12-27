#!/bin/bash
set -e

sudo apt-get install -y libsdl2-dev
cargo build
