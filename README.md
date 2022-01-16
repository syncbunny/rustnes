# cppnes
A NES Enulator written in Rust

## NOT Supported features
+ Second controller (and mic-in)
+ DMC Sound
+ PAL Mode (Only NTSC MOde is supported).

## Supported mappers
None(only mapper-zero is supported).

## Supported platform
+ MacOS + SDL2
+ Ubuntu 21.10 + SDL2

## Branches
+ develop: master branch

## How to build
cargo b --release

## Usage
rustnes <ROMFile>
  
## Controll (Pad-1)
+ UP/Down/Left/Right: Cursor keys
+ Start: return key
+ Select: right shift key
+ A: 'X' key
+ B: 'Z' key
