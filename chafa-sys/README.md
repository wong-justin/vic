# chafa-sys

Rust bindings for [`chafa`](https://hpjansson.org/chafa/), a C library that makes pretty pictures in the terminal. 

## Usage

Install `chafa` and its dependency [`glib`](https://docs.gtk.org/glib/), either from your package manager or from source.
Then put this crate in your Rust project.

Typical usage:

```toml
[dependencies]
chafa-sys = { features = ["link-dynamic"] }
``` 

<!-- for an example of a rust project using chafa-sys, see [vic]. -->

### 1) Give library locations if necessary

By default, build.rs will call [pkg-config](https://people.freedesktop.org/~dbn/pkg-config-guide.html) to try to find `chafa` and its dependencies.
If libraries are not found, you can provide library names and directories with `$RUSTFLAGS`.

Example: `RUSTFLAGS="-L /usr/lib -l chafa -l glib" cargo build`

This may be useful when:

- libraries are built or installed to non-standard locations
- libraries are not built with pkg-config metadata
- building in environments with unique linking needs, like when compiling with musl libc
- building on Windows? or environments that do not support pkg-config

<!-- metion .a or .so files? -->

### 2) Choose dynamic or static linking

This crate has two cargo build features: `["link-dynamic"]` or `["link-static"]`.
You must choose one.

Static builds have been trickier in my experience, mainly due to `glib` compilation errors.
I was able to make a statically linked build in an Alpine [Dockerfile].
Other people have statically linked `chafa` & `glib` in other environments -- see [vic issue #1](https://github.com/wong-justin/vic/issues/1) to explore those options.

## This crate does not:

- build `chafa` or `glib` from source (yet).
- test builds on Windows or macOS (yet).

If you need those things, I recommend modifying `build.rs` or writing your own containerized build script.
If you can vendor `chafa` and `glib` and build them from source in `build.rs`, or if you have changes that make the build work on Windows or macOS, or if you have usability suggestions in general, your contribution would be appreciated.

## Licenses

`chafa` uses LGPL-3.0-or-later.

`glib` uses LGPL-2.1-or-later.
