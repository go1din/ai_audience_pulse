# BaselHack 2025

This repository contains the results of **Freestyle Challenge Group 3.1**.
The idea was to create a realtime pulse radar of a crowd to detect reactions, facial expressions & general consensus of a crowd in a presentation.
Initally the idea was to run everything on a Seeed ESP32S Sense that has a camera and a microphone but we extended the scope a little bit to increase the quality of the product.
We still made 3 ESP32 running though to in the end have 3 ESPs plus the host camera and microphone working.
The frontend combines all video and audio signals and displays general signals in a timeline and in realtime. 

![thumbsup](./assets/thumbup.jpg)

## How to run the web parts

You need python, uv & zx on a "modern" unix.

Ubuntu setup has been tested, uv & zx installation instructions are [here](code/README.md#install).
OSX works as well and has been tested.
We dont take responsibility for running it on Windows. 

Check out [readme.md](code/README.md) for proper setup instructions.

## How to run the remote esp32 cameras

see [Camera readme](code/camera/esp32s3/README.md)


