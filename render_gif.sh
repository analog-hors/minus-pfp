#!/usr/bin/env sh
rm -f frames/*.png
cargo run --release && ffmpeg -framerate 30 -i "frames/%05d.png" out.gif -y
