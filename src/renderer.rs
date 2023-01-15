use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use crate::output::Output;

#[derive(Debug)]
pub struct StandardRenderer {
    out: Arc<Mutex<Output>>,
}

struct Notifier {}

impl StandardRenderer {
    pub fn new() -> StandardRenderer {
        StandardRenderer {
            out: Arc::new(Mutex::new(Output::new())),
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = channel();
        self.interval_refresh(tx);
        let out = self.out.clone();
        listen(out, rx);
    }

    pub fn quit(&mut self) {
        let mut o = self.out.lock().unwrap();
        o.quit();
    }

    fn interval_refresh(&self, tx: Sender<Notifier>) {
        thread::spawn(move || loop {
            tx.send(Notifier {}).unwrap();
        });
    }

    pub fn write(&mut self, new_data: &str) {
        let out = self.out.clone();
        let out = out.lock();
        if let Ok(mut o) = out {
            o.write(new_data);
        }
    }
}

fn listen(out: Arc<Mutex<Output>>, rx: Receiver<Notifier>) {
    thread::spawn(move || {
        while rx.recv().is_ok() {
            if let Ok(mut o) = out.lock() {
                o.flush();
            }
        }
    });
}
