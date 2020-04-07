# 4d-game

<img src="https://raw.githubusercontent.com/akuczala/4d-game/master/screenshot-3d.png" width="400"> <img src="https://raw.githubusercontent.com/akuczala/4d-game/master/screenshot-4d.png" width="400">

First person 4d geometry game engine, written in Rust, inspired by www.urticator.net. Under development; the current version amounts to a demo.

There is a 3d and a 4d mode, toggled with backspace.

## Controls

Currently uses flying tank controls for movement:
- <kbd>W</kbd>/<kbd>S</kbd> : Move forward/back
- <kbd>A</kbd>/<kbd>D</kbd> : Turn left/right (yaw)
- <kbd>I</kbd>/<kbd>K</kbd> : Turn up/down (pitch)
- <kbd>J</kbd>/<kbd>K</kbd> : (In 4d) turn other left/right
- <kbd>SHIFT</kbd>+<kbd>J</kbd>/<kbd>K</kbd> : Rolls player in 3d, rotates lateral axes in 4d

- Holding down <kbd>ALT</kbd> causes the player to slide along an axis instead of rotate.

- <kbd>Backspace</kbd> Switches between 3d/4d mode
- <kbd>C</kbd> Toggles clipping

## To do:
- update to newest version of winit
- Faster rendering pipeline
- Collisions
- Level design and spatial challenges


