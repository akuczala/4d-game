# 4d-game

<img src="https://raw.githubusercontent.com/akuczala/4d-game/master/screenshot-3d.png" width="400"> <img src="https://raw.githubusercontent.com/akuczala/4d-game/master/screenshot-4d.png" width="400">

First person 4d geometry game engine, written in Rust, inspired by www.urticator.net. Under development; the current version amounts to a demo.

There is a 3d and a 4d mode, toggled with backspace.

## Controls

Your choice of FPS-like controls or tank controls, toggled with <kbd>M</kbd>
### FPS controls
Move with <kbd>WASD</kbd> in 3d and <kbd>QWEASD</kbd> in 4d. Mouse controls will be familiar in 3d. In 4d, moving the mouse will turn the player along the two lateral axes. When holding <kbd>SHIFT</kbd>, moving the mouse will spin the lateral axes and allow the player to look up and down.

### Tank controls
- <kbd>W</kbd>/<kbd>S</kbd> : Move forward/back
- <kbd>A</kbd>/<kbd>D</kbd> : Turn left/right (yaw)
- <kbd>I</kbd>/<kbd>K</kbd> : Look up/down (pitch)
- <kbd>Q</kbd>/<kbd>E</kbd> : (In 4d) turn other left/right
- <kbd>SHIFT</kbd>+<kbd>Q</kbd>/<kbd>E</kbd> : Rotates lateral axes in 4d

- Holding down <kbd>ALT</kbd> causes the player to slide along an axis instead of rotate.

- <kbd>Backspace</kbd> Switches between 3d/4d mode
- <kbd>C</kbd> Toggles clipping

## Edit mode
Edit mode can be enabled from the config file.
Click on an object to select it.
- <kbd>F</kbd> : Freely manipulate object with player controls

- <kbd>T</kbd> : Translate mode
- <kbd>R</kbd> : Rotate mode
- <kbd>Y</kbd> : Scale mode
- <kbd>M</kbd> : Confirm change and exit edit mode
- <kbd>\\</kbd> : Discard change and exit edit mode

Use - <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd>, <kbd>4</kbd> to select axes, and scroll or move the mouse or to manipulate the selected object. Translation requires at least one axis, and rotation requires 2 or 4. Holding <kbd>~</kbd> snaps movement to discrete intervals.
- <kbd>SHIFT</kbd>+<kbd>~</kbd>: Snap object to closest half-integer grid point.
- <kbd>,</kbd>: Reset object orientation and scale.
- <kbd>/</kbd>: Delete object
- <kbd>;</kbd>: Duplicate object
- <kbd>.</kbd>: Create cube


## New in 0.2.0
- config file
- fuzz texture
- skybox
- edit mode

## To do:
- Level design and spatial challenges


