## Bare Metal Space Invaders
A Space Invaders game built for RPI3 running without any operating system.


## Space invaders module
this crate can run both with std using xx and on bare metal by implementing the right interfaces.

## FrameBuffer
top left corner is (0,0) and x increases going to the right, and y increases going down.

## Commands
* a: move left
* d: move right
* space: shoot
* r: restart game

---
maybe:
* alien ship in foreground top of the screen
* animations


## How to run it
You can run it from your desktop using:
```
cargo run --package space_invaders --bin space_invaders --features std
```
Or if you want to run it on your Raspberry pi, follow the steps:

1. install Raspberry Pi OS (ex Raspbian) to an sd card.
2. run "./build.sh" to build a kernel8.img binary file.
2. Replace kernel8.img in the sd with the one you just built.
3. Connect the usb serial output to Raspberry pi like the image below. Connect the HDMI as well.
4. Connect the usb to your laptop and wait

Check https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials#-usb-serial-output for additional guidance.