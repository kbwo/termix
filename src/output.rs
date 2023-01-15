use std::io::{BufRead, Stdout, Write};

use crate::{
    cursor::{detect_cursor_pos, CursorPos},
    raw::{IntoRawMode, RawTerminal},
};

trait Flushable {
    fn flush();
}

pub struct Output {
    buf: Vec<u8>,
    out_target: RawTerminal<Stdout>,
    start_pos: CursorPos,
    lines: usize,
    inited: bool,
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
        let start_pos = detect_cursor_pos(&mut raw).unwrap();

        Output {
            buf: vec![],
            out_target: raw,
            lines: 0,
            start_pos,
            inited: false,
        }
    }
    pub fn write(&mut self, new_data: &str) {
        self.buf.extend(new_data.as_bytes());
    }

    pub fn flush(&mut self) {
        if !self.buf.is_empty() {
            // avoid buffer overflow
            let start_y = self.start_pos.0
                - if self.start_pos.0 >= self.lines {
                    self.lines
                } else {
                    self.start_pos.0
                };
            let _ = self
                .out_target
                .write(format!("\x1b[{};{}H", start_y, self.start_pos.1).as_bytes());
            let _ = self.out_target.write(b"\x1b[0J");
            if self.buf.lines().count() < self.lines {
                self.start_pos.0 -= self.lines - self.buf.lines().count();
            }
            self.lines = std::str::from_utf8(&self.buf)
                .unwrap_or("")
                .replace("\x1b[0m", "")
                .lines()
                .count();
        }
        let _ = self.out_target.write(&self.buf);
        self.buf.clear();
        let _ = self.out_target.flush();
        self.inited = true;
    }

    pub fn quit(&mut self) {
        self.out_target.finish_raw().unwrap();
    }
}
