//!
//! ## Termix
//! Termix is a framework to build TUI application with simplicity.
//! This framework is inspired by [bubbletea](https://github.com/charmbracelet/bubbletea).
//!
//!
//! For now, this framework is beta version and WIP.
//!
//! ## Usage
//!
//! In your `Cargo.toml` add the following:
//!
//! ```toml
//! [dependencies]
//! termix = "0.1.0"
//! ```
//!
//! Here is an example:
//!
//! ```no_run
//! use std::time::Duration;
//!
//! use termix::{
//!     color::{Color, StyledText},
//!     event::Event,
//!     model::{ModelAct, Updater},
//!     Program,
//! };
//!
//! struct Model(usize);
//!
//! #[derive(Debug)]
//! struct Tick {}
//!
//! impl ModelAct<Model, Tick> for Model {
//!     fn update(&self, event: &Event<Tick>) -> Updater<Model, Tick> {
//!         match event {
//!             Event::Init => (Some(Box::new(Model(self.0))), Some(tick)),
//!             Event::Custom(_) => {
//!                 if self.0 - 1 == 0 {
//!                     return (Some(Box::new(Model(self.0 - 1))), Some(|| Event::Quit));
//!                 }
//!                 (Some(Box::new(Model(self.0 - 1))), Some(tick))
//!             }
//!             Event::Keyboard(..) => (None, Some(|| Event::Quit)),
//!             _ => (None, None),
//!         }
//!     }
//!     fn view(&self) -> String {
//!         StyledText::new(
//!             &format!(
//!                 "Hi. This program will exit in {} seconds. To quit sooner press any key.\n",
//!                 self.0
//!             ),
//!             Some(Color::Ansi256(212)),
//!             None,
//!             None,
//!             None,
//!             None,
//!         )
//!         .text()
//!     }
//! }
//!
//! fn tick() -> Event<Tick> {
//!     let one_sec = Duration::from_secs(1);
//!     std::thread::sleep(one_sec);
//!     Event::Custom(Tick {})
//! }
//!
//! fn main() {
//!     Program::new(Box::new(Model(5))).run();
//! }
//! ```
//!
//! To know how to use termix practically, you can look at the examples
//!
pub mod color;
mod cursor;
mod error;
pub mod event;
mod input;
pub mod key;
pub mod model;
mod output;
mod raw;
mod renderer;

use model::ModelAct;
use std::{
    fmt::Debug,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use event::Event;
use input::KeyBoard;
use renderer::StandardRenderer;

pub struct Program<T, E: Send + Debug> {
    renderer: Arc<Mutex<StandardRenderer>>,
    event_tx: Sender<Event<E>>,
    event_rx: Arc<Receiver<Event<E>>>,
    model: Arc<Mutex<Box<dyn ModelAct<T, E>>>>,
}

impl<T, E: Send + Debug + 'static> Program<T, E> {
    /// Initialize models and internals.
    pub fn new(model: Box<dyn ModelAct<T, E>>) -> Program<T, E> {
        let (e_tx, e_rx) = channel();
        Program {
            renderer: Arc::new(Mutex::new(StandardRenderer::new())),
            event_tx: e_tx,
            event_rx: Arc::new(e_rx),
            model: Arc::new(Mutex::new(model)),
        }
    }

    /// Starts UI and event loop
    pub fn run(&mut self) {
        self.renderer.lock().unwrap().start();
        let key_tx = self.event_tx.clone();
        thread::spawn(move || start_key_listener(key_tx));
        let tx = self.event_tx.clone();
        tx.send(Event::Init).unwrap();
        self.event_loop();
    }

    fn event_loop(&mut self) {
        let rx = self.event_rx.clone();
        while let Ok(ev) = rx.recv() {
            match &ev {
                Event::Quit => {
                    self.renderer.lock().unwrap().quit();
                    break;
                }
                _ => {
                    if let Ok(mut model) = self.model.lock() {
                        let (new_model, cmd) = model.update(&ev);
                        if let Some(new) = new_model {
                            let _ = std::mem::replace(&mut *model, new);
                            self.renderer.lock().unwrap().write(&model.view());
                        }
                        let tx = self.event_tx.clone();
                        thread::spawn(move || {
                            if let Some(cmd) = cmd {
                                let ev = cmd();
                                tx.send(ev).unwrap();
                            }
                        });
                    }
                }
            }
        }
    }
}

fn start_key_listener<E: Send + Debug>(event_tx: Sender<Event<E>>) {
    let mut keyboard = KeyBoard::new();
    while let Ok(key) = keyboard.next_key_timeout(Duration::from_secs(0)) {
        event_tx.send(Event::Keyboard(key.clone())).unwrap();
    }
}
