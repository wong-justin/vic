// vic
//
// important functions: init(), update(), view()
//
// ffmpeg decodes video frames to pixel data on stdout
// chafa converts pixel frames into terminal-encoded ANSI/CSI/block/unicode/ascii graphics
// then we add video editing controls/state
//
// requires:
// - libchafa, dynamically linked (since my static build isn't working)
// - ffmpeg on $PATH, tested with version 3.4.8
// - some rust dependencies like crossterm
//
// ----------------------------------------
// lesser TODOs:
//
// i should use chafa::canvas.print_rows() so i dont have to undo line work
//
// replace all .ok_or(format!(...)) with .ok_or_else(|| format!(...))
// see: https://medium.com/@techhara/rust-tip-and-trick-2-5a92641191f6
//
// use generic Pathlike P instead of filepath: String
// and figure out generic type signatures everywhere
//
// consider using mut &str for model.frame instead of String.
// not sure if this will actually help performance
//
// ----------------------------------------
//
// LEARNINGS:
//
// - can't declare Box[] of large size because stack isnt that big.
// instead, declare vec[] and then .into_boxed_slice() will avoid initializing on stack
//
// - "Slices are similar to arrays, but their length is not known at compile time.
// Instead, a slice is a two-word object; the first word is a pointer to the data,
// the second word is the length of the slice."
//
// - slice = &array[start .. endplusone]
//
// - "Arrays are stack allocated."
// https://doc.rust-lang.org/rust-by-example/primitives/array.html
//
// - "Writing more than a pipe buffer’s worth of input to stdin without also reading stdout and
// stderr at the same time may cause a deadlock."
//
// - Vec seems faster than box. idk why.
//
// - looping read_exact() 30x seems faster than reading ffmpeg output stream into a buffer 30x as
// large. idk why.
//
// - not sure how to seek backwards except for caching frames/data after reading it
//
// - maybe framereading process can keep reading as long as possible, while display process will
// only request frames less frequently
//
// - making a new vec buffer each render call seems nearly as fast
// as using one permanently assigned vec, surprisingly
//
// - giving the decoding process a head start and caching first few frames might also help it feel
// smoother
//
// - internally, chafa represents each char as 8x8 bitmap.
// so a canvas of 1 row of 2 chars has 8x8 internal resolution
// see: chafa/internal/chafa-symbols-block.h
//
// - a random data point on my machine:
// ffmpeg decoding 60fps 180p video into raw frames into /dev/null takes
// ~3.5 mins, with 1.5GB/sec throughput, and results in 115GB of raw pixel frame output
//
// - usize is meant for referencing memory locations. so dont use it for typical application
// properties.
//
// perhaps learn more from:
// https://github.com/jart/hiptext/blob/master/src/movie.cc
// https://github.com/zmwangx/rust-ffmpeg/blob/master/examples/dump-frames.rs
// https://github.com/maxcurzi/tplay/blob/main/src/pipeline/frames.rs
// https://gist.github.com/AndreaCatania/2b708750ef62171f51c7038e99676822
// https://github.com/oddity-ai/video-rs
// https://medium.com/init-deep-dive/rust-video-frame-extraction-speed-comparison-4d33fcc99405
// chafa_src/tools/chafa/chafa.c::line2963+ while (is_animation)
// libavformat
// libavutil

#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::error::Error;
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crossterm::{
    cursor::{MoveLeft, MoveTo, MoveToColumn, MoveToNextLine, MoveToPreviousLine, MoveToRow},
    event::{read as await_next_event, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use log::info;

mod tui;
use crate::tui::{Program, UpdateResult};
mod chafa;
use crate::chafa::{Canvas, Config, SymbolMap, Symbols};

// --- LOGS --- //

struct LoggerToFile<'a> {
    path: &'a str,
}

impl log::Log for LoggerToFile<'_> {
    fn flush(&self) {}
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&self.path)
                //
                // Not sure what to do when log file fails.
                // Either silently fail or crash. I choose crash
                .expect("failure to open or read log file");
            writeln!(file, "{}", record.args());
        }
    }
}

static LOGGER: LoggerToFile = LoggerToFile {
    // TODO: maybe $LOCALAPPDATA/vic/log for windows,
    // and maybe $HOME/.vic/log for linux, or that $XDG_HOME thing
    path: "/tmp/vic_log",
};
const DOWNSCALE_FACTOR: f64 = 0.5; // 0.125;
const NUM_COLOR_CHANNELS: i32 = 3;
const NUM_FRAMES_TO_TRACK_FPS: u8 = 10; // arbitrary interval to recalculate fps

// --- MODEL, and other data structures --- //

struct Model {
    terminal_cols: Columns,
    terminal_rows: Rows,
    VIDEO_METADATA: VideoMetadata, // const, but not enforcable since Model is mutable
    frame_iterator: FrameIterator,
    frame: String,

    // a 24hr video could have 60fps * 60sec/min * 60min/hr * 24hrs = 5_184_000 frames,
    // which is safely between     u16::MAX = 65_536  and  u32::MAX = 4_294_967_296 frames
    frame_number: u32,

    paused: bool,
    markers: Vec<SecondsFloat>,
    speed: f32,
    hovering: Hovering, // current marker or segment
    hide_controls: bool,
    needs_to_clear: bool, // if screen resized, we should clear janky screen artifacts

    // for calculating next frame number
    prev_instant: std::time::Instant,
    accumulated_time: SecondsFloat,

    // for displaying fps
    recent_fps: Option<f64>,
    last_fps_check: std::time::Instant,

    // maybe have derived attributes like:
    //   frame_number -> position_millis,
    //   duration_secs -> duration_millis,
    //   markers<SecondsFloat> -> markers<FrameNumber>,
    //   markers -> segments,
    //   duration_secs -> max_frame_number
    // so computation is not repeated

    // for debugging, to check time elapsed since beginning
    start: std::time::Instant,
}

enum HoverMode {
    Markers,
    Segments,
}

struct Hovering {
    mode: HoverMode,
    position: usize, // an index in a vec of markers/segments
}

type SecondsFloat = f64; // to indicate when we're using units of time

// note that crossterm uses u16, like in terminal::size() or MoveToColumn()
// and that chafa uses i32, like in canvas.set_geometry()
//
// assuming terminal size is under u16::MAX = 65_535u16 cols/lines
type Columns = u16;
type Rows = u16;

struct FrameIterator {
    canvas: chafa::Canvas,
    video_path: String,   // ideally path: P or &str, but String is just easier
    input_width: i32,     // pixels
    input_height: i32,    // pixels
    output_cols: Columns, // aka chars
    output_rows: Rows,    // aka lines
    stdout: std::process::ChildStdout, // pixels get piped to here
    // stderr: std::process::ChildStderr,
    // cur_frame_number: u32,
    pixel_buffer: Vec<u8>,
    num_frames_rendered: u32, // for debugging
}

struct VideoMetadata {
    width: i32,  // pixels
    height: i32, // pixels
    fps: f64,
    duration_secs: SecondsFloat,
    // maybe max_frame_number: u32, // derived from duration and fps, for convenience
    seconds_per_frame: SecondsFloat, // derived from fps, for convenience
}

fn get_ffprobe_video_metadata(video_filepath: &str) -> Result<VideoMetadata, Box<dyn Error>> {
    // note/TODO:
    // https://stackoverflow.com/q/6239350#comment92617706_6239379
    // if metdata is corrupted, ffprobe will report wrong data, but ffmpeg will have correct data
    // so maybe dont use ffprobe
    //
    // to learn:
    // https://ffmpeg.org//ffprobe.html#Main-options
    // https://stackoverflow.com/a/24488789
    //
    // typical ffprobe output is a few sections:
    //
    // [FORMAT]
    // filename=abc
    // nb_streams=1
    // duration=...
    // size=...
    // [/FORMAT]
    // [STREAM]
    // duration=...
    // avg_frame_rate=...
    // nb_frames=...
    // [/STREAM]
    // [STREAM]
    // ...
    // [/STREAM]
    //
    // but it's reformattable, ie. -print_format

    let probe_process = std::process::Command::new("ffprobe")
        // verbosity. i dont think this matters here
        // .args(["-v", "error"])
        //
        // only need the first [STREAM] (ie. don't need secondary streams like audio)
        .args(["-select_streams", "v:0"])
        .args([
            "-show_entries",
            //
            // get the width, height, and r_frame_rate properties from the [STREAM] section
            // and get the duration property from the [FORMAT] section
            //
            // note that [FORMAT]::duration seems reliable,
            // while [STREAM]::duration is not reliable
            //
            // also note: the output could be in any order
            // https://ffmpeg.org//ffprobe.html#:~:text=the%20order%20of%20specification%20of%20the%20local%20section%20entries%20is%20not%20honored%20in%20the%20output%2C%20and%20the%20usual%20display%20order%20will%20be%20retained.
            "stream=width,height,r_frame_rate:
            format=duration",
        ])
        //
        .args([
            "-print_format",
            // some -print_formats include default, json, and csv/compact
            // we use compact; ignore the [STREAM/FORMAT] section labels; and separate with commas
            //
            // https://ffmpeg.org//ffprobe.html#default
            "compact=
            print_section=0:item_sep=,",
        ])
        .arg(&video_filepath)
        .output()
        .map_err(|e| format!("ffprobe process failed {}", e))?;

    let plain_output = String::from_utf8(probe_process.stdout)?;
    let err = String::from_utf8(probe_process.stderr)?;

    // expecting cmd_output in the format:
    // ```
    // width=643,height=528,r_frame_rate=30/1
    // duration=ss.microseconds
    // ```
    // so replace newlines with commas
    // then split on commas
    // to get:
    // ```
    // key=val,key=val,key=val
    // ```

    let single_line_output = plain_output.trim().replace('\n', ",");
    log::info!("{}\n{}", single_line_output, err);

    let mut ffprobe_properties = std::collections::HashMap::<&str, &str>::new();

    for line in single_line_output.split(',') {
        let (key, val) = line
            .split_once("=")
            .ok_or(format!("failed to split part of ffprobe output {} -- expected key=val. ffprobe command probably failed", line))
            .map_err(|e| format!("failed to parse part of ffprobe output {}", line))?;

        ffprobe_properties.insert(key, val);
    }

    // parse keyval map of strings into typed struct

    let width = ffprobe_properties
        .get("width")
        .ok_or(format!("failed to get width {}", plain_output))?
        .parse::<i32>()
        .map_err(|e| format!("failed to parse {} {}", e, plain_output))?;
    let height = ffprobe_properties
        .get("height")
        .ok_or(format!("failed to get height {}", plain_output))?
        .parse::<i32>()
        .map_err(|e| format!("failed to parse {} {}", e, plain_output))?;
    let (frames, per_second) = ffprobe_properties
        .get("r_frame_rate")
        .ok_or(format!("failed to get frame rate {}", plain_output))?
        .split_once('/')
        .ok_or(format!("failed to parse fps {}", plain_output))?;
    let fps = (frames
        .parse::<f64>()
        .map_err(|e| format!("failed to parse {} {}", e, plain_output))?)
        / (per_second
            .parse::<f64>()
            .map_err(|e| format!("failed to parse {} {}", e, plain_output))?);
    let duration_secs = ffprobe_properties
        .get("duration")
        .ok_or(format!("failed to get duration {}", plain_output))?
        .parse::<SecondsFloat>()
        .map_err(|e| format!("failed to parse {} {}", e, plain_output))?;

    log::info!("{} {} {} {}", width, height, fps, duration_secs);

    return Ok(VideoMetadata {
        width: width,
        height: height,
        fps: fps,
        seconds_per_frame: 1.0 / fps,
        duration_secs: duration_secs,
    });
}

impl FrameIterator {
    fn _create_decoding_process(
        video_filepath: &str,
        start_time: SecondsFloat,
    ) -> Result<std::process::ChildStdout, Box<dyn Error>> {
        // init long-running ffmpeg decoding process.
        // this is where a lot of the heavy lifting happens.
        // ffmpeg must be available on $PATH.
        // tested with ffmpeg version 3.4.8-ubuntu... built with gcc 7

        let mut process = std::process::Command::new("ffmpeg")
            .args(["-ss", &format!("{:0<3}", start_time)])
            .args(["-i", &video_filepath])
            // .args(["-nostdin"]) -nostdin perhaps solves: https://stackoverflow.com/a/47114881
            //
            // rgb24 = 8:8:8 bytes, r:g:b
            // this is a straightforward format, and chafa accepts it, so let's just always use it
            .args(["-pix_fmt", "rgb24"])
            .args(["-f", "rawvideo"])
            // downscaling the video vastly improves performance
            .args([
                "-vf", // aka "-filter_complex",
                &format!("scale=iw*{}:ih*{}", DOWNSCALE_FACTOR, DOWNSCALE_FACTOR),
            ])
            //
            // maybe also try the arg max_muxing_queue_size='9999' ?
            //
            .args(["pipe:"])
            .stdout(std::process::Stdio::piped())
            //
            // problem with .stderr(std::process::Stdio::piped()), if I don't consume it:
            // stderr will eventually reach pipe capacity if not consumed,
            // so the program will hang after ~270 secs or ~65536 bytes of ffmpeg stderr output.
            //
            // to read more descriptions of the problem:
            // 0) https://wonger.dev/posts/chafa-ffmpeg-progress#ffmpeg-recipes
            // 1) https://github.com/rust-lang/rust/issues/45572#issuecomment-860134955
            // 2) https://github.com/oconnor663/duct.py/blob/master/gotchas.md#using-io-threads-to-avoid-blocking-children
            // 3) https://docs.python.org/2/library/subprocess.html#subprocess.call#:~:text=not%20use%20stdout%3DPIPE%20or%20stderr%3DPIPE%20with%20this%20function%20as%20that%20can%20deadlock%20based%20on%20the%20child%20process%20output%20volume.
            //
            // Solutions:
            // - use a lib like `duct` that uses background threads to prevent stdout and stderr
            // from reaching pipe capacity,
            // - copy that solution of threading into my program, or
            // - redirect stderr to /dev/null, so it will always be consumed and never reach
            // pipe capacity
            //
            // We'll use the /dev/null solution for now because it's the easiest.
            // Ideally, we capture stderr and monitor it for error messages.
            // But ffmpeg poops out line-by-line logs next to important plaintext error msgs
            // So I think it would be challenging to parse stderr.
            // And there's more important features/fixes to focus on for now.
            //
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| "ffmpeg decoding process failed")?;

        log::info!("{}", "created ffmpeg decoding process");

        let stdout = process
            .stdout
            .take()
            .ok_or("failed to take stdout from ffmpeg decoding process")?;

        return Ok(stdout);
    }

    fn new(
        video_filepath: String,
        input_width: i32,
        input_height: i32,
        output_cols: Columns,
        output_rows: Rows,
        blocky: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let stdout = FrameIterator::_create_decoding_process(&video_filepath, 0.0)?;

        // --- CHAFA CONFIG --- //

        let symbol_map = chafa::SymbolMap::new();
        symbol_map.add_by_tags(match blocky {
            // SOLID = 1 symbol, full height block, which is ugly
            // VHALF = 2 symbols, commonly used by other image2ansi libraries
            // HALF = 4 symbols, horizontal or vertical half, just as ugly as SOLID
            true => chafa::Symbols::VHALF,
            //
            // 29 + 11 + 97 ~= 140 symbols, fast enough and pretty enough
            // TODO: maybe remove border symbols from this combo?
            // since block + geometric looks decent,
            // and there are a lot of border symbols (eg. noticeable performance impact)
            false => chafa::Symbols::BLOCK | chafa::Symbols::GEOMETRIC | chafa::Symbols::BORDER,
            //
            // false => chafa::Symbols::ALL, // ~600 symbols, very slow
        });
        let config = chafa::Config::new();
        config.set_geometry(output_cols as i32, output_rows as i32);
        config.set_symbol_map(symbol_map);
        config.set_work_factor(1.0);
        //
        // TODO: check bindings to make sure chafa enums carry over properly
        // these other canvas color modes aren't working for me
        config.set_canvas_mode(chafa::CanvasMode::TRUECOLOR); // chafa::CanvasMode::INDEXED_8/256/16,
        let canvas = chafa::Canvas::new(config);

        return Ok(Self {
            canvas: canvas,
            video_path: video_filepath,
            input_width: input_width,
            input_height: input_height,
            output_cols: output_cols,
            output_rows: output_rows,
            stdout: stdout,
            pixel_buffer: vec![0u8; (input_width * input_height * NUM_COLOR_CHANNELS) as usize],
            num_frames_rendered: 0,
        });
    }

    fn take_frame(&mut self) -> String {
        self.stdout.read_exact(&mut self.pixel_buffer);
        self.num_frames_rendered += 1;

        self.canvas.draw_all_pixels(
            chafa::PixelType::RGB8,
            &self.pixel_buffer,
            self.input_width,
            self.input_height,
            (self.input_width * NUM_COLOR_CHANNELS) as i32,
        );

        let view_string = self.canvas.build_ansi();
        return view_string;
    }

    fn skip_some_frames(&mut self, num_frames: u32) -> String {
        // When you only want to advance a few frames,
        // without spawning a new ffmpeg process,
        // but also without calling chafa.draw each frame as you would in loop { take_frame() }
        //
        // If skipping many frames,
        // you should probably just start a new ffmpeg process with .goto_timestamp()
        for _ in 0..num_frames {
            self.stdout.read_exact(&mut self.pixel_buffer);
        }
        return self.take_frame();
    }

    fn goto_timestamp(&mut self, timestamp: SecondsFloat) -> Result<String, Box<dyn Error>> {
        // Start new process at any position in video.
        // This should be faster than reading far ahead in the old process,
        // and this enables "backward seeking" too.

        let new_stdout = FrameIterator::_create_decoding_process(&self.video_path, timestamp)?;
        self.stdout = new_stdout;
        Ok(self.take_frame())
    }
}

#[derive(Debug)]
struct CliArgs {
    video_filepath: String,
    max_width: Columns,
    hide_controls: bool,
    //
    // secret options for now; placeholders for future
    muted: bool,
    blocky: bool,
    just_the_recipe: bool,
}

// TODO: custom app events
// enum Action {
//     A,
//     B,
// }
//
// TODO: custom errors and better error handling
// enum VicError {
//     BadFfprobe,
//     BadCliArg,
// }

// --- INIT --- //

fn init() -> Result<Model, String> {
    let HELP_MSG: String = format!(
        "
 vic {} - cut videos in the terminal

 _____
 USAGE

   vic <filepath> [-w <int, default 40>]
                  [--hide-controls]
                  [--help|--version]
 ________
 EXAMPLES

   vic video.mp4
   vic video.mp4 -w=9999 --hide-controls
   vic http://example.com/video.avi -w 20

 _______
 OPTIONS

   -w <int>          Max output width, in columns.
                     Use -w 9999 for fullscreen.
                     Defaults to 40.
   --hide-controls   Hide helper text below the video.

 ________
 CONTROLS

   [ segment mode ]

     space ... play/pause
     j/l ..... seek back/forwards
     s ....... remove/keep segment
     m ....... make marker
     q ....... finish

   [ marker mode ]

     M ....... delete marker
     J/L ..... goto prev/next marker

 _____
 NOTES

   vic accepts video filepaths that ffmpeg accepts, including URLs.
   vic fails if the video has no known duration,
   which may occur in corrupted or incomplete video files.
   vic needs at least 14 columns.

   source: github.com/wong-justin/vic

",
        // examples: ffmpeg -i bigvideo.mp4 -vf scale="iw/4:ih/4" smallvideo.mp4 \
        //   && vic small_video.mp4 --just-the-recipe
        env!("CARGO_PKG_VERSION")
    );

    // Receive command-line args.
    // https://github.com/RazrFalcon/pico-args/blob/master/examples/app.rs
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP_MSG);
        std::process::exit(0);
    }
    if pargs.contains(["-v", "--version"]) {
        print!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    let args = CliArgs {
        video_filepath: pargs
            .free_from_str::<std::path::PathBuf>()
            // Note: commands like `vic -w 20 video.mp4` will fail, but not on this error,
            // because pargs always assumes arg[1] is a valid positional arg.
            // This is a limitation of the pargs library.
            // Positional args must be listed first, like `vic video.mp4 -w 20`
            .map_err(|e| "failed to parse command-line arg <filepath>")?
            .display()
            .to_string(),
        max_width: pargs
            .opt_value_from_fn("-w", Columns::from_str)
            .map_err(|e| "failed to parse -w")?
            .unwrap_or(40),
        muted: pargs.contains("--muted"),
        hide_controls: pargs.contains("--hide-controls"),
        blocky: pargs.contains("--blocky"),
        just_the_recipe: pargs.contains("--just-the-recipe"),
    };
    log::info!("{:?}", args);

    // Init app state.
    let (cols, rows): (Columns, Rows) =
        terminal::size().map_err(|e| format!("failed to get terminal size {}", e))?;

    let video_metadata = get_ffprobe_video_metadata(&args.video_filepath)
        .map_err(|e| format!("failed to get video metadata {}", e))?;
    let fps = video_metadata.fps;

    let aspect_ratio = video_metadata.width as f64 / video_metadata.height as f64;
    let output_cols = std::cmp::min(cols - 2, args.max_width - 2) as Columns;
    let output_rows = (output_cols as f64 / aspect_ratio / 2.0).ceil() as Rows;
    log::info!("{:?} {:?} {:?}", aspect_ratio, output_cols, output_rows);

    let frame_iterator = FrameIterator::new(
        args.video_filepath.to_string(),
        (video_metadata.width as f64 * DOWNSCALE_FACTOR) as i32,
        (video_metadata.height as f64 * DOWNSCALE_FACTOR) as i32,
        output_cols,
        output_rows,
        args.blocky,
    )
    .map_err(|e| format!("failed to initialize video reader {}", e))?;

    let model = Model {
        paused: false,
        frame_number: 0,
        speed: 1.0,
        markers: Vec::<SecondsFloat>::new(),
        hovering: Hovering {
            mode: HoverMode::Segments,
            position: 0,
        },
        terminal_cols: cols,
        terminal_rows: rows,
        VIDEO_METADATA: video_metadata,
        frame_iterator: frame_iterator,
        hide_controls: args.hide_controls,
        frame: "".to_string(),
        needs_to_clear: false,
        prev_instant: std::time::Instant::now(),
        last_fps_check: std::time::Instant::now(),
        recent_fps: None,
        start: std::time::Instant::now(),
        accumulated_time: 0.0,
    };

    // enum TimerEvent {}

    // terminal_event_broadcaster = new_thread.spawn(forever { await_next_terminal_event() })
    // timer_broadcaster = timer_thread.spawn(forever { every(100ms).ping() })
    //
    // here.watch(broadcaster).on_broadcast(|| {
    //     update(model, AppEvent::ShowNextFrame);
    // })
    // here.watch(terminal_event_broadcaster).on_broadcast(|event| {
    //     update(model, event);
    // })
    //
    // maybe tokio::join!{task1, task2}
    // or
    // tokio::select!{task1, task2}
    // https://tokio.rs/tokio/tutorial/select

    // let forever = tokio::spawn(async {
    //     let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
    //     loop {
    //         interval.tick().await;
    //         model.frame = model.frame_iterator.take_frame();
    //     }
    // });

    // also a way to sleep forever maybe: std::future::pending().await;
    return Ok(model);
}

// --- UPDATE --- //

fn update(m: &mut Model, terminal_event: Event) -> UpdateResult {
    m.needs_to_clear = false;
    match terminal_event {
        Event::Key(keyevent) => {
            if (keyevent.modifiers == KeyModifiers::CONTROL && keyevent.code == KeyCode::Char('c'))
                || keyevent.code == KeyCode::Char('q')
            {
                return UpdateResult::Finish;
            }

            match keyevent.code {
                KeyCode::Char(' ') => toggle_paused(m),
                KeyCode::Char('h') => toggle_controls_visibility(m),
                KeyCode::Char('j') => seek_backwards_15s(m),
                KeyCode::Char('l') => seek_forwards_15s(m),
                KeyCode::Char('J') => goto_prev_marker(m),
                KeyCode::Char('L') => goto_next_marker(m),
                KeyCode::Char('m') => create_marker(m),
                KeyCode::Char('M') => delete_marker(m),
                KeyCode::Char('.') => advance_one_frame(m),
                _ => (),
            };
        }
        Event::Resize(cols, rows) => {
            m.terminal_cols = cols;
            m.terminal_rows = rows;
            m.needs_to_clear = true;
        }
        _ => (),
    };

    let now = std::time::Instant::now();

    if m.paused {
        m.prev_instant = now;
        return UpdateResult::Continue;
    }

    let whole_elapsed_frames = frames_since_prev_instant(m);
    let is_too_fast = whole_elapsed_frames == 0;
    match is_too_fast {
        true => {
            // slow down by not drawing anything during this tick
            m.prev_instant = now;
            return UpdateResult::Continue;
        },
        false => {
            // now we know the next frame number to render
            m.frame = m.frame_iterator.skip_some_frames(whole_elapsed_frames - 1);
            m.frame_number += whole_elapsed_frames as u32;
        }
    }

    // --- update stats --- //

    let time_to_update_fps =
        m.frame_iterator.num_frames_rendered % NUM_FRAMES_TO_TRACK_FPS as u32 == 0;
    if time_to_update_fps {
        let recent_time_elapsed: SecondsFloat = (now - m.last_fps_check).as_secs_f64();
        m.recent_fps = Some(NUM_FRAMES_TO_TRACK_FPS as f64 / recent_time_elapsed);
        m.last_fps_check = now;
    } else {
        // Either too early and not enough recent data,
        // or waiting for next moment to check fps.
        // No need to constantly update fps every frame.
    }

    let time_to_log = m.frame_iterator.num_frames_rendered % 100 == 1;
    if time_to_log {
        log::info!(
            "{:?} {:?} {:?} {:?}",
            now - m.start,
            m.prev_instant,
            m.frame_iterator.num_frames_rendered,
            m.frame_number
        );
    }

    m.prev_instant = now;
    return UpdateResult::Continue;
}

fn frames_since_prev_instant(m: &mut Model) -> u32 {
    // find how many frames elapsed since last tick,
    // and modify leftover time

    // example converting elapsed ms to elapsed frames:
    //
    // 0.040 secs elapsed   30 frames   1.2 frames
    // ------------------ * --------- =   elapsed
    //                       1 second
    //
    // whole_frames_elapsed = 1 frame
    //
    //                0.2 frames    1 second
    // rounding_err = ---------- * --------- = 0.007 seconds
    //                             30 frames
    //
    let now = std::time::Instant::now();
    let elapsed_secs = (now - m.prev_instant).as_secs_f64();

    // how many frames should have passed since last tick; sometimes 0, usually 1 or more
    let mut whole_elapsed_frames: u32 = (elapsed_secs * m.VIDEO_METADATA.fps).floor() as u32;

    // account for accumulated deltas from rounding down.
    // the leftover time eventually adds up to frame's worth of compensated time
    // (like an extra day in a leap year)
    let rounding_err: SecondsFloat =
        elapsed_secs - (whole_elapsed_frames as f64 * m.VIDEO_METADATA.seconds_per_frame);
    m.accumulated_time += rounding_err;

    let need_leap_frame = m.accumulated_time > m.VIDEO_METADATA.seconds_per_frame;
    if need_leap_frame {
        whole_elapsed_frames += 1;
        m.accumulated_time -= m.VIDEO_METADATA.seconds_per_frame;
    }

    // log::info!("{:?} {:?} {:?} {:?}",
    //            elapsed_secs,
    //            whole_elapsed_frames,
    //            extra_time,
    //            m.accumulated_time);

    return whole_elapsed_frames;
}

fn toggle_paused(m: &mut Model) {
    m.paused = !m.paused;
    if !m.paused {
        m.hovering.mode = HoverMode::Segments;
    }
}

fn toggle_controls_visibility(m: &mut Model) {
    m.hide_controls = !m.hide_controls;
    log::info!("hide controls? {:?}", m.hide_controls);
    m.needs_to_clear = true; // controls will still show at bottom unless cleared/drawn over
}

// note: any function that modifies playerhead position aka m.frame_number in Segment mode
// must check if segment number has changed

fn seek_backwards_15s(m: &mut Model) {
    let frames_to_backtrack = (m.VIDEO_METADATA.fps * 15.0) as u32;
    m.frame_number = std::cmp::max(m.frame_number as i32 - frames_to_backtrack as i32, 0) as u32;
    let timestamp = m.frame_number as SecondsFloat / m.VIDEO_METADATA.fps;
    m.frame = m.frame_iterator.goto_timestamp(timestamp).unwrap();

    // TODO: update current segment
    // let old_position = m.hovering.position;
    // let preceding_markers
    // while let Some(other_timestamp) = preceding_markers.next() {
}

fn seek_forwards_15s(m: &mut Model) {
    let frames_to_skip = (m.VIDEO_METADATA.fps * 15.0) as u32;
    m.frame_number += frames_to_skip;
    let timestamp = m.frame_number as SecondsFloat / m.VIDEO_METADATA.fps;
    m.frame = m.frame_iterator.goto_timestamp(timestamp).unwrap();

    // TODO: update current segment
    // let upcoming_markers =
}

fn goto_prev_marker(m: &mut Model) {
    // enter marker mode and goto nearest backwards timestamp
    //
    // segment    0     1     2
    //         ┌─────┬─────┬──────┐
    //         └─────┴─────┴──────┘
    // marker        0     1
    //
    let new_position = m.hovering.position as i32 - 1;
    match new_position >= 0 {
        false => (),
        true => {
            m.hovering = Hovering {
                mode: HoverMode::Markers,
                position: new_position as usize,
            };
            let timestamp: SecondsFloat = m.markers[new_position as usize];
            m.frame_number = (timestamp * m.VIDEO_METADATA.fps) as u32;
            m.frame = m.frame_iterator.goto_timestamp(timestamp).unwrap();
            m.paused = true;
        }
    }
}

fn goto_next_marker(m: &mut Model) {
    // enter marker mode and goto nearest forwards timestamp
    //
    // segment    0     1     2
    //         ┌─────┬─────┬──────┐
    //         └─────┴─────┴──────┘
    // marker        0     1
    //
    let new_position = match m.hovering.mode {
        HoverMode::Markers => m.hovering.position + 1,
        HoverMode::Segments => m.hovering.position,
    };
    match new_position < m.markers.len() {
        false => (),
        true => {
            m.hovering = Hovering {
                mode: HoverMode::Markers,
                position: new_position,
            };
            let timestamp: SecondsFloat = m.markers[new_position];
            m.frame_number = (timestamp * m.VIDEO_METADATA.fps) as u32;
            m.frame = m.frame_iterator.goto_timestamp(timestamp).unwrap();
            m.paused = true;
        }
    };
}

fn create_marker(m: &mut Model) {
    // create new marker at current timestamp
    match m.hovering.mode {
        HoverMode::Markers => (),
        HoverMode::Segments => {
            let timestamp: SecondsFloat = m.frame_number as SecondsFloat / m.VIDEO_METADATA.fps;
            m.markers.push(timestamp);
            log::info!("{:.3}", timestamp);
        }
    }
}

fn delete_marker(m: &mut Model) {
    // delete current marker and enter segments mode
    match m.hovering.mode {
        HoverMode::Segments => (),
        HoverMode::Markers => {
            m.markers.remove(m.hovering.position);
            m.hovering.mode = HoverMode::Segments;
        }
    }
}

fn advance_one_frame(m: &mut Model) {
    match m.paused {
        false => (),
        true => {
            m.frame = m.frame_iterator.skip_some_frames(1);
            m.frame_number += 1;
        }
    }
}

// --- VIEW --- //

fn format_secs_to_mm_ss(seconds: SecondsFloat) -> String {
    let minutes = (seconds / 60.0).floor();
    let remaining_secs = (seconds - (minutes * 60.0)).floor();
    return format!("{}:{:0>2}", minutes, remaining_secs);
}

fn view(m: &Model, outbuf: &mut impl std::io::Write) {
    // --- empty cases and setup --- //

    if m.needs_to_clear {
        queue!(outbuf, terminal::Clear(terminal::ClearType::All));
    }

    queue!(outbuf, MoveTo(0, 0),);
    // a lot of ugly syntax just to make an empty placeholder frame
    let mut lines: Box<dyn std::iter::Iterator<Item = &str>> = match m.frame.is_empty() {
        false => Box::new(m.frame.split("\n")),
        true => Box::new(std::iter::repeat("").take(m.frame_iterator.output_rows as usize)),
    };

    // --- draw colorful video frame and overlaid labels --- //
    //
    // TODO: fix flickering labels overlayed on video.
    // Currently, terminal emulator prints colorful video and then backtracks to print text.
    // need to parse line of video output, which looks like
    // 38;2;r;g;b;48;2;r;g;b;  38;2;r;g;b;48;2;r;g;b; ... [0m
    //
    // also consider using new-age control character for synchornized terminal render update,
    // created by ghostty/zig people

    let first_line = lines.next().unwrap();
    queue!(
        outbuf,
        Print(" "),
        Print(first_line),
        // hardcoded template width, assuming double digit fps
        MoveLeft(8),
        Print(match m.recent_fps {
            Some(fps) => format!(" fps: {:2.0}", fps),
            None => " fps:   ".to_string(),
        }),
        MoveToNextLine(1),
    );
    for line in lines {
        queue!(outbuf, Print(" "), Print(line), MoveToNextLine(1),);
    }
    queue!(
        outbuf,
        MoveToPreviousLine(1),
        Print(" "),
        Print(format!(
            "{} / {} ",
            format_secs_to_mm_ss(m.frame_number as f64 / m.VIDEO_METADATA.fps),
            format_secs_to_mm_ss(m.VIDEO_METADATA.duration_secs)
        )),
        MoveToColumn(m.frame_iterator.output_cols - 1),
        // Print(match m.paused {
        Print(match m.paused {
            true => " ||",
            false => " >>",
            // TODO:
            // true => "paused || x2 / x1 / x0.5",
            // false => "playing >> x2 / x1 / x0.5",
            // buffering => "buffering 6-frame loading cycle of braille ⠆⠃⠉⠘⠰⠤
        }),
        MoveToNextLine(1),
    );

    // --- draw playerbar and stats --- //
    //
    //  1:04.567 / 1:23                 x1  >>
    // ┏-----------┳--v--┳━━━━━━━━━━━━━━━━━━━━┓
    // ┗-----------┻-----┻━━━━━━━━━━━━━━━━━━━━┛
    //  segment 2 of 3
    //
    //  0:56.789 / 1.23         x2 ||
    // ┌-----------v-----┬───────────┐
    // └-----------┴-----┴───────────┘
    //  marker 1 of 2

    let percent_complete =
        (m.frame_number as f64 / m.VIDEO_METADATA.duration_secs / m.VIDEO_METADATA.fps) * 100.0;
    let playerhead_position =
        (percent_complete * m.frame_iterator.output_cols as f64 / 100.0) as Columns;

    queue!(
        outbuf,
        Print(format!(
            "┌{}┐",
            "─".repeat(m.frame_iterator.output_cols as usize)
        )),
        MoveToNextLine(1),
        Print(format!(
            "└{}┘",
            "─".repeat(m.frame_iterator.output_cols as usize)
        )),
        MoveToPreviousLine(1)
    );
    for timestamp in &m.markers {
        let position = (m.frame_iterator.output_cols as f64
            * (timestamp / m.VIDEO_METADATA.duration_secs)) as Columns;
        queue!(
            outbuf,
            MoveToColumn(position),
            Print("┬"),
            MoveToNextLine(1),
            MoveToColumn(position),
            Print("┴"),
            MoveToPreviousLine(1)
        );
    }
    queue!(
        outbuf,
        MoveToColumn(playerhead_position),
        Print("v"),
        MoveToNextLine(2),
    );

    if m.hide_controls {
        return;
    }

    // --- draw helper text / controls --- //
    //
    //                          segment 2 of 3
    //     m = make marker
    //   J/L = prev/next marker
    //     s = keep segment
    // space = pause
    //   j/l = skip back/forwards
    //     q = finish, making 1 segment

    let num_markers = m.markers.len();
    let num_segments = num_markers + 1;

    queue!(
        outbuf,
        MoveToColumn(m.frame_iterator.output_cols - 12),
        Print(match m.hovering.mode {
            HoverMode::Segments =>
                format!("segment {} of {}", m.hovering.position + 1, num_segments),
            // HoverMode::Segments => format!("     {} segments", num_segments),
            HoverMode::Markers => format!(" marker {} of {}", m.hovering.position + 1, num_markers),
        }),
        MoveToNextLine(1),
    );

    queue!(
        outbuf,
        Print(match m.hovering.mode {
            HoverMode::Segments => "     m = make marker           ",
            HoverMode::Markers => "     M = remove marker         ",
        }),
        MoveToNextLine(1),
        Print("   J/L = prev/next marker    "),
        MoveToNextLine(1),
        // TODO: if m.current_segment.is_removed, then text = "keep segment"
        // Print("     s = remove segment        "),
        // MoveToNextLine(1),
        Print(match m.paused {
            true => " space = unpause",
            false => " space = pause  ",
        }),
        MoveToNextLine(1),
        Print("   j/l = skip back/forwards  "),
        MoveToNextLine(1),
        Print("     h = hide controls       "),
        MoveToNextLine(1),
        Print(match num_segments {
            1 => "     q = quit                          ".to_string(),
            _ => format!("     q = quit and cut into {} segments", num_segments),
        }),
        MoveToNextLine(1),
    );
}

// --- APP START --- //

// #[tokio::main]
fn main() {
    log::set_logger(&LOGGER).map(|_| log::set_max_level(log::LevelFilter::Info));

    log::info!(
        "\n--- new session, version {} ---",
        env!("CARGO_PKG_VERSION")
    );

    let program_result = Program { init, view, update }.run();
    match program_result {
        Ok(mut m) => {
            if m.markers.len() == 0 {
                return;
            }

            // bookend markers with implicit start and end timestamps
            m.markers.insert(0, 0.);
            m.markers.push(m.VIDEO_METADATA.duration_secs);
            let mut iter_markers = m.markers.iter();
            let mut start = iter_markers.next().unwrap();
            let mut i = 0;

            // /a/b/c.mp4 becomes /a/b/c_0.mp4, /a/b/c_1.mp4, ...
            //
            // Since this is the end of the application,
            // I'm wagering that an err wouldve thrown by now if video_path
            // was missing its parent, extension, or file stem. so just unwrap().
            let filepath = std::path::PathBuf::from(&m.frame_iterator.video_path);
            let outdir = filepath.parent().unwrap();
            let extension = filepath.extension().unwrap().to_str().unwrap();
            let filename = filepath.file_stem().unwrap().to_str().unwrap();

            while let Some(end) = iter_markers.next() {
                log::info!("trimming from {} to {}", start, end);
                let process = std::process::Command::new("ffmpeg")
                    .arg("-ss")
                    .arg(format!("{:0<3}", start))
                    .arg("-i")
                    .arg(&m.frame_iterator.video_path)
                    .arg("-to")
                    .arg(format!("{:0<3}", end))
                    //
                    // TODO: confirm if -c copy uses millisecond-precision
                    // ie, will it ruin frame-perfect cuts?
                    .arg("-c")
                    .arg("copy")
                    .arg(outdir.join(format!("{}_{}.{}", filename, i, extension)))
                    .stdout(std::io::stdout())
                    .stderr(std::io::stderr())
                    .output()
                    .map_err(|_| "ffmpeg cutting process failed");

                start = end;
                i += 1;
            }

            std::process::exit(0);
        }
        Err(msg) => {
            // app failed; explain why
            log::info!("Error: {}", msg.to_string());
            write!(std::io::stderr(), "Error: {}", msg.to_string());
            std::process::exit(1);
        }
    };
}
