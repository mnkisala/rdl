[![Crates.io][crates-badge]][crates-url]

[crates-badge]: https://img.shields.io/crates/v/rdl.svg
[crates-url]: https://crates.io/crates/rdl

# Rusty Dmenu Launcher

The entire purpose of this utility is to read .desktop entries
and pipe them to something like [dmenu](https://tools.suckless.org/dmenu/)
or [rofi](https://github.com/davatorium/rofi).

I made this thing as a replacement for [j4-dmenu-desktop](https://github.com/enkore/j4-dmenu-desktop)
(which at the time simply didn't work on my machine for some reason), which in turn
is a replacement for [i3-dmenu-desktop](https://i3wm.org/).

# Usage

Simply running `rdl` will open up dmenu with desktop entries
located in `/usr/share/applications`. You can set your dmenu
command, terminal for terminal applications and paths where .desktop
files are located with command line parameters.

# Config

Check out [the example config](exampleconfig.yaml).

# Installation

## From Github

```
git clone https://github.com/mnkisala/rdl.git
cd rdl
cargo install --path .
```

## From Crates.io

```
cargo install rdl
```
