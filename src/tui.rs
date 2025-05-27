// mini functional framework for application lifecycle and terminal display

#![allow(unused_variables)]
#![allow(unused_imports)]

use std::fs::DirEntry;
use std::io::Write;
use std::path::{Path, PathBuf};

use crossterm::{
    cursor::MoveTo,
    event::{read as await_next_event, Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Print,
    terminal,
};

use std::{io::stdout, time::Duration};

pub struct Program<Init, View, Update> {
    pub init: Init,
    pub view: View,
    pub update: Update,
}

pub enum UpdateResult {
    Continue,
    Finish,
    Failed(String),
}

impl<Init, View, Update> Program<Init, View, Update> {
    // #[tokio::main]
    pub fn run<Model>(self) -> Result<Model, String>
    where
        Init: FnOnce() -> Result<Model, String>,
        View: Fn(&Model, &mut std::io::Stderr),
        // update() mutates the model bc I think it's a bit easier and more performant
        //   than creating a new Model in memory on each update
        //   although maybe returning Model { newfield: _, ..oldmodel } would work fine
        Update: Fn(&mut Model, Event) -> UpdateResult,
    {
        let Self { init, view, update } = self;
        // write all TUI content to stderr, so other tools can parse stdout on finish
        let mut stderr = std::io::stderr();

        let mut model = init()?; // quit early here if init fails

        // disable some behavior like line wrapping and catching Enter presses
        // because i will handle those myself
        // https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode
        terminal::enable_raw_mode();
        queue!(
            stderr,
            terminal::EnterAlternateScreen,
            terminal::Clear(terminal::ClearType::All), // sometimes term/tmux bugs out and doesnt clear alternate screen in time before we start, so just clear it to be sure
            terminal::DisableLineWrap,
            crossterm::cursor::Hide,
        );

        view(&model, &mut stderr);
        stderr.flush();

        // --- init tasks / threads / asyncs / event loop(s) --- //

        // let (timer_event_sender, rx1) = tokio::sync::mpsc::channel(1);
        // let (terminal_event_sender, rx2) = tokio::sync::mpsc::channel(1);

        // tokio::spawn(async move {
        //     let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
        //     loop {
        //         interval.tick().await;
        //         log::info!("writing from inside timer tokio task");
        //         timer_event_sender.send(());
        //     }
        // });

        // tokio::spawn(async move {
        //     loop {
        //         // await next timer event from other thread too
        //         let event = await_next_event().unwrap();
        //         log::info!("writing from inside terminal event tokio task");
        //         terminal_event_sender.send(());
        //     }
        // });

        // tokio::select! {
        //     val = rx1 => {
        //         println!("rx1 completed first with {:?}", val);
        //     }
        //     val = rx2 => {
        //         println!("rx2 completed first with {:?}", val);
        //     }
        // }

        // let mut reader = EventStream::new();
        // log::info!("{:?}", reader);

        // loop {
        //     tokio::select! {
        //         _ = async { loop {
        //             tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        //             log::info!("writing from inside timer tokio task");
        //         }} => {},
        //         _ = async { loop {
        //             tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        //             log::info!("writing from inside 2second timer tokio task");
        //         }} => {},
        //         _ = async {
        //             crossterm::non_blocking_read...
        //             log::info!("received terminal event");
        //         } => {},
        //     }
        // }

        // synchronous method 1)
        //
        loop {
            // just poll often for terminal events
            //
            // while still letting cpu rest a little with this sleep() call.
            // note that 33ms would be enough for 30fps if each frame is processed instantly
            //
            // also note/TODO: this constant polling keeps cpu active even when video is paused.
            // the alternative is overhauling and refactoring to implement async
            // (im not ready for that work yet) (esp since i want to avoid heavy tokio dependency)
            if crossterm::event::poll(Duration::from_millis(16)).unwrap() {
                // 200, 100, 66, 33, 16
                let event = crossterm::event::read().unwrap();
                match update(&mut model, event) {
                    UpdateResult::Continue => (),
                    UpdateResult::Finish => break,
                    UpdateResult::Failed(msg) => {
                        return Err(msg);
                        () // to satisfy compiler
                    }
                };
            } else {
                // no terminal event happened during last N millis.
                //
                // send dummy event, just to trigger a redraw
                // TODO: create a proper 'next tick' event
                update(
                    &mut model,
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('n'),
                        modifiers: KeyModifiers::NONE,
                    }),
                );
            }

            view(&model, &mut stderr);
            stderr.flush();
        }

        // synchronous method 2)
        //
        // loop {
        //     let event = await_next_event().unwrap(); // blocking
        //     match update(&mut model, event) {
        //         UpdateResult::Continue => (),
        //         UpdateResult::Finish => break,
        //         UpdateResult::Failed(msg) => {
        //             return Err(msg);
        //             () // to satisfy compiler return type
        //         }
        //     };
        //
        //     view(&model, &mut stderr);
        //     stderr.flush();
        // }

        // cleanup and be a good citizen so the terminal behaves normally afterwards (eg. start catching ctrl+c again, and show cursor)
        //
        // TODO: cleanup even after panic
        execute!(
            stderr,
            terminal::EnableLineWrap,
            terminal::LeaveAlternateScreen,
            crossterm::cursor::Show,
        );
        terminal::disable_raw_mode();
        Ok(model)
    }
}
