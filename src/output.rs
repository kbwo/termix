use std::io::{Stdout, Write};

use crate::raw::{IntoRawMode, RawTerminal};

trait Flushable {
    fn flush();
}

pub struct Output {
    buf: Vec<u8>,
    out_target: RawTerminal<Stdout>,
    lines: usize,
}
unsafe impl Send for Output {}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("buf", &self.buf)
            .field("out_target", &"hidden".to_string())
            .finish()
    }
}

impl Output {
    pub fn new() -> Output {
        let mut raw = std::io::stdout().into_raw_mode().unwrap();
        // hide cursor
        raw.hide_cursor().unwrap();
        raw.flush().unwrap();

        Output {
            buf: vec![],
            out_target: raw,
            lines: 0,
        }
    }
    pub fn write(&mut self, new_data: &str) {
        self.buf.extend(new_data.as_bytes());
    }

    pub fn flush(&mut self) {
        if !self.buf.is_empty() {
            for _ in 0..self.lines {
                let _ = self.out_target.write(b"\x1bM");
            }
            let _ = self.out_target.flush();
            let _ = self.out_target.write(b"\x1b[0J");
            let _ = self.out_target.flush();
            self.lines = std::str::from_utf8(&self.buf)
                .unwrap_or("")
                .replace("\x1b[0m", "")
                .lines()
                .count();
        }
        let _ = self.out_target.write(&self.buf);
        self.buf.clear();
        let _ = self.out_target.flush();
    }

    pub fn quit(&mut self) {
        self.out_target.finish_raw().unwrap();
    }
}
