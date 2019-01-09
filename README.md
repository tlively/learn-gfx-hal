[![License:Apache2](https://img.shields.io/badge/License-Apache2-green.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/39wvbxxstqjd2vi8?svg=true)](https://ci.appveyor.com/project/Lokathor/learn-gfx-hal)
[![travis.ci](https://travis-ci.org/Lokathor/learn-gfx-hal.svg?branch=master)](https://travis-ci.org/Lokathor/learn-gfx-hal)

![gfx-hal:0.1](https://img.shields.io/badge/gfx--hal-0.1-blue.svg)

# learn-gfx-hal

Step by step tutorials for using the `gfx-hal` crate.

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
