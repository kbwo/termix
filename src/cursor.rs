use std::io::Read;
use std::{io::Stdout, os::fd::AsRawFd};

use crate::error::TermixError;
use crate::input::wait_until_ready;
use crate::raw::{get_tty, RawTerminal};
use std::{io::Write, sync::mpsc::channel, thread, time::Duration};

struct Prepared;

#[derive(Debug)]
pub struct CursorPos(pub usize, pub usize);

pub fn detect_cursor_pos(stdout: &mut RawTerminal<Stdout>) -> Result<CursorPos, TermixError> {
    let timeout = Duration::from_secs(0);
    let mut prepared = false;
    let (tx, rx) = channel();
    let (buf_tx, buf_rx) = channel::<[u8; 7]>();
    thread::spawn(move || {
        let mut tty = get_tty();
        tx.send(Prepared).unwrap();
        wait_until_ready(tty.as_raw_fd(), None, timeout).unwrap(); // wait timeout
        let mut reader_buf = [0; 7];
        tty.read_exact(&mut reader_buf).unwrap();
        buf_tx.send(reader_buf).unwrap();
    });
    while rx.recv().is_ok() && !prepared {
        prepared = true;
        stdout.lock().write_all("\x1B[6n".as_bytes()).unwrap();
        stdout.flush().unwrap();
    }
    let mut cursor: Option<CursorPos> = None;

    while let Ok(buf) = buf_rx.recv() {
        let mut read_str = std::str::from_utf8(&buf).unwrap().to_string();
        read_str.pop();
        let beg = read_str.rfind('[').unwrap();
        let coords: String = read_str.chars().skip(beg + 1).collect();
        let mut nums = coords.split(';');

        let cy = nums.next().unwrap().parse::<usize>().unwrap_or(0);
        let cx = nums.next().unwrap().parse::<usize>().unwrap_or(0);
        cursor = Some(CursorPos(cy, cx));
    }
    cursor.ok_or(TermixError::CursorDetection)
}

#[cfg(test)]
mod tests {
    use crate::raw::IntoRawMode;

    use super::detect_cursor_pos;

    #[test]
    fn cursor_pos() {
        // test only if in common device
        let _ = std::io::stdout().into_raw_mode().map(|mut stdout| {
            let pos = detect_cursor_pos(&mut stdout);
            assert!(pos.is_ok());
        });
    }
}
