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
    let delimiter = b'R';
    let (tx, rx) = channel();
    let (buf_tx, buf_rx) = channel::<Vec<u8>>();
    thread::spawn(move || {
        let mut buf: [u8; 1] = [0];
        let mut read_chars = Vec::new();
        let mut tty = get_tty();
        tx.send(Prepared).unwrap();
        wait_until_ready(tty.as_raw_fd(), None, timeout).unwrap(); // wait timeout
        while buf[0] != delimiter {
            if tty.read(&mut buf).unwrap() > 0 {
                read_chars.push(buf[0]);
            }
        }
        buf_tx.send(read_chars).unwrap();
    });
    if rx.recv().is_ok() {
        stdout.lock().write_all("\x1B[6n".as_bytes()).unwrap();
        stdout.flush().unwrap();
    }
    let mut cursor: Option<CursorPos> = None;

    if let Ok(buf) = buf_rx.recv() {
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
