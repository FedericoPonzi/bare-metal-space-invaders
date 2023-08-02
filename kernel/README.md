## Kernel crate
This is used to interface with the hardware. It's a no_std, no_main project that is tested on Rasbpi3b+. It might work on newer models as well, but it hasn't been tested there.

Can be tested by running make build, make chainboot and make qemu.

1. install Raspberry Pi OS (ex Raspbian) to an sd card.
2. Replace kernel8.img in the sd with the one provided in the assets folder (assets/kernel8-chainboot.img) in the repo.
3. Connect the usb serial output to Raspberry pi like the image below. Connect the HDMI as well.
4. Run `make chainboot`.
5. Connect the usb to your laptop and wait

Check https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials#-usb-serial-output for additional guidance.