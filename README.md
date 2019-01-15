[![License:Apache2](https://img.shields.io/badge/License-Apache2-green.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/39wvbxxstqjd2vi8?svg=true)](https://ci.appveyor.com/project/Lokathor/learn-gfx-hal)
[![travis.ci](https://travis-ci.org/Lokathor/learn-gfx-hal.svg?branch=master)](https://travis-ci.org/Lokathor/learn-gfx-hal)

[![gfx-hal:0.1](https://img.shields.io/badge/gfx--hal-0.1-blue.svg)](https://docs.rs/gfx-hal)

# learn-gfx-hal

Step by step tutorials for using the [gfx-hal](https://github.com/gfx-rs/gfx)
crate.

The tutorials all target the current version _on crates.io_, not on the master
branch of git. At the time of me writing this that means `0.1.0`.

This is _not_ intended to be a library crate for you to just import into your
own projects. It is a series of examples and explanations for you read and learn
from.

* Fully working examples are in the `examples/` directory. Each example attempts
  to be a single file that works on its own, so in some cases the code style
  _isn't_ quite what you'd want on a full project (eg: shader code contained in
  string literals instead of saved in separate files).
* The lessons are in the `book/` directory in markdown form. They can be
  rendered to HTML with [mdbook](https://github.com/rust-lang-nursery/mdBook),
  and the [GitHub Pages site](https://lokathor.github.io/learn-gfx-hal/) for
  this repository hosts a rendered version of the master branch.

## Requirements

Uses [shaderc-rs](https://github.com/google/shaderc-rs), please follow [their
setup instructions](https://github.com/google/shaderc-rs#setup).

* Regarding `msys2` on Windows: Note that the _first_ time you run the `pacman`
  command they list _it doesn't install the packages_. Instead it actually just
  installs the latest pacman and msys files. You have to then close that window
  entirely and open a new one, then run the `pacman` command again to make it
  actually download the stuff.
* The following steps assume you installed to `C:\msys64`. If not, adjust
  accordingly:
  * When they say "the msys2 mingw64 binary path" they mean `C:\msys64\usr\bin`
    and `C:\msys64\mingw64\bin`.
  * Open a command prompt as Administrator and make a symbolic link for
    `python3.exe` to also be seen as `python.exe`
    * `mklink C:\msys64\mingw64\bin\python.exe C:\msys64\mingw64\bin\python3.exe`
* Yes, everything I just told you to do is totally stupid, but it is also real
  advice that you must follow in this, the year of our lord two thousand and
  nineteen, if you want to program 3D graphics programs.

## Contribution

This repo is Apache 2 licensed and all of your contributions must be made under
that license.

## Disclaimer

Hello. The primary ~~author~~ quixotic fool of this project is me, Lokathor. I
explain things in a way that beginners can hopefully understand by not actually
knowing anything about it myself. I get advice and guidance from the
[gfx-rs](https://github.com/gfx-rs) team when writing each lesson, but the final
editorial sign off is mine, and and crazy or stupid opinions that you find here
are _mine alone_, not any fault of theirs.

If there are mistakes that have crept in anywhere please file an issue. You can
also attempt to contact me interactively on the `#gamedev` channel of the
[community discord](https://bit.ly/rust-community).
