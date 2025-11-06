# vic

Play & cut videos in the terminal

![screenshot](https://github.com/user-attachments/assets/da6770b1-2595-477e-98a4-94e95450912e)

## Building

### Linux

`vic` is dynamically linked with [`chafa`](https://hpjansson.org/chafa/), a C library that makes pretty pictures. 
Install `chafa` from your package manager, or build it from source:

```
apt-get install libglib2.0-dev
curl 'https://hpjansson.org/chafa/releases/chafa-1.14.4.tar.xz' -O
tar xf chafa-1.14.4.tar.xz
cd chafa-1.14.4
./configure --without-tools
make
make install
ldconfig
```

Once `chafa` is installed, you can build the Rust project with `cargo build`.

Make sure everything is compiled and linked correctly by running `cargo test`.

You can find the built binary at `target/debug/vic`, or install it with `cargo install --path .`

`vic` requires [`ffmpeg`](https://ffmpeg.org//download.html) to be on `$PATH` during runtime.

### Static binaries

Coming eventually! 
See [this issue](https://github.com/wong-justin/vic/issues/1#issue-2586904982) if you want to help.

## Examples

```
vic video.mp4
vic video.mp4 -w=9999
vic http://example.com/video.avi -w 20
vic video.webm -w 80 --dry-run
```

## Usage

```
vic <filepath> [-w <int, default 40>]
               [--dry-run]
               [--help|--version]
```

### Options

```
-w <int>          Max output width, in columns.
                  Use -w 9999 for fullscreen.
                  Defaults to 40.

--dry-run         Instead of auto-running ffmpeg commands
                  on finish, just print the commands to stdout.
```

### Controls

```
[ segment mode ]

  space ... play/pause
  j/l ..... seek back/forwards
  0-9 ..... seek to 0%, 10%, etc
  . ....... advance one frame
  m ....... make marker
  q ....... finish

[ marker mode ]

  M ....... delete marker
  J/L ..... goto prev/next marker
```

## Notes

Here's a blog post: https://wonger.dev/posts/chafa-ffmpeg-progress

My main focus for now is creating a separate UI thread and adding audio.

I also need to fiddle with GitHub Actions and building static binaries.

There's several quality-of-life improvements to work on.

Pull requests welcome :)

Created during [LMT2](https://lmt2.com/).
