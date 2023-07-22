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

# TODO:
* refactoring: framebuffer
* shots rate limit.

---
maybe:
* alien ship in foreground top of the screen
* animations
* 