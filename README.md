# vic

Play & cut videos in the terminal

[demo.webm](https://github.com/user-attachments/assets/89d099d0-21fe-482b-b793-03fa053c79ad)

## Building

### Linux

`vic` is dynamically linked with [`chafa`](https://hpjansson.org/chafa/), a C library that makes pretty pictures. To install `chafa`:

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

You can find the built binary at `target/debug/vic`, or you can use `cargo run` as an alias for `vic`.

`vic` requires [`ffmpeg`](https://ffmpeg.org//download.html) to be on `$PATH` during runtime.

## Static binaries

Coming soon! See [this issue](https://github.com/wong-justin/vic/issues/1#issue-2586904982) if you want to help.

## Usage

```
vic <filepath> [-w <int, default 40>]
               [--hide-controls]
               [--help|--version]
```

### Examples

```
vic video.mp4
vic video.mp4 -w=9999 --hide-controls
vic http://example.com/video.avi -w 20
```

### Options

```
-w <int>          Max output width, in columns.
                  Use -w 9999 for fullscreen.
                  Defaults to 40.

--hide-controls   Hide helper text below the video.
```

### Controls

```
[ segment mode ]

  space ... play/pause
  j/l ..... seek back/forwards
  m ....... make marker
  q ....... finish

[ marker mode ]

  M ....... delete marker
  J/L ..... goto prev/next marker
```

## Notes

Here's a blog post: https://wonger.dev/posts/chafa-ffmpeg-progress

My main focus for now is fiddling with GitHub Actions and building static binaries.

I also need to use an async runtime. Right now, the program gets sluggish with large videos, and it interferes with user input.

Another big task is to add audio.

There's several quality-of-life improvements to work on.

Pull requests welcome :)
