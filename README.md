# `polar-arctic`
[![Build and Test](https://github.com/BeaconBrigade/polar-arctic/actions/workflows/rust.yml/badge.svg)](https://github.com/BeaconBrigade/polar-arctic/actions/workflows/rust.yml)

`polar-arctic` is a cross platform application for reading heart rate, ECG and acceleration data from the Polar H10.
GUI is made using [iced](https://github.com/iced-rs/iced), [plotters](https://github.com/plotters-rs/plotters) and 
[plotters-iced](https://github.com/Joylei/plotters-iced). The back-end is built using [arctic](https://github.com/Roughsketch/arctic)

# Usage

Use `cargo install` to get this binary and run it from anywhere. **Note**: when specify file paths for output, all paths are interpreted relatively. For example the path `~/ecg.csv`
will literally look for a directory titled `~`, which it won't find and it will crash. Similarly, you can't use `/` to start at the root of the file system. Use the menu, and data screen help buttons
for more information. 
