use nix::fcntl::fcntl;
use nix::fcntl::FcntlArg;
use nix::fcntl::OFlag;
use nix::sys::select;
use nix::sys::time::TimeVal;
use nix::sys::time::TimeValLike;
use std::os::unix::prelude::FromRawFd;
use std::os::unix::prelude::RawFd;
use std::time::Duration;
use std::{fs::File, io::Read, os::unix::prelude::AsRawFd};

use crate::error::TermixError;
use crate::key::Key;
use crate::raw::get_tty;

const KEY_WAIT: Duration = Duration::from_millis(10);
fn duration_to_timeval(duration: Duration) -> TimeVal {
    let sec = duration.as_secs() * 1000 + (duration.subsec_millis() as u64);
    TimeVal::milliseconds(sec as i64)
}

pub fn wait_until_ready(
    fd: RawFd,
    signal_fd: Option<RawFd>,
    timeout: Duration,
) -> Result<(), TermixError> {
    let mut timeout_spec = if timeout == Duration::new(0, 0) {
        None
    } else {
        Some(duration_to_timeval(timeout))
    };

    let mut fdset = select::FdSet::new();
    fdset.insert(fd);
    if let Some(fd) = signal_fd {
        fdset.insert(fd)
    }
    let n = select::select(None, &mut fdset, None, None, &mut timeout_spec)
        .map_err(|_| TermixError::KeyListener)?;

    if n < 1 {
        Err(TermixError::KeyListener) // this error message will be used in input.rs
    } else if fdset.contains(fd) {
        Ok(())
    } else {
        Err(TermixError::KeyListener) // this error message will be used in input.rs
    }
}

pub struct KeyBoard {
    file: Box<File>,
    // bytes will be poped from front, normally the buffer size will be small(< 10 bytes)
    pub byte_buf: Vec<u8>,
    sig_rx: File,
    next_key: Option<Key>,
}
impl Default for KeyBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBoard {
    pub fn new() -> KeyBoard {
        let file = get_tty();
        // the self-pipe trick for interrupt `select`
        let (rx, _tx) = nix::unistd::pipe().expect("failed to set pipe");

        // set the signal pipe to non-blocking mode
        let flag = fcntl(rx, FcntlArg::F_GETFL).expect("Get fcntl failed");
        let mut flag = OFlag::from_bits_truncate(flag);
        flag.insert(OFlag::O_NONBLOCK);
        let _ = fcntl(rx, FcntlArg::F_SETFL(flag));

        // set file to non-blocking mode
        let flag = fcntl(file.as_raw_fd(), FcntlArg::F_GETFL).expect("Get fcntl failed");
        let mut flag = OFlag::from_bits_truncate(flag);
        flag.insert(OFlag::O_NONBLOCK);
        let _ = fcntl(file.as_raw_fd(), FcntlArg::F_SETFL(flag));
        KeyBoard {
            file,
            sig_rx: unsafe { File::from_raw_fd(rx) },
            byte_buf: vec![],
            next_key: None,
        }
    }

    #[allow(dead_code)]
    pub fn next_key(&mut self) -> Result<Key, TermixError> {
        self.next_key_timeout(Duration::new(0, 0))
    }

    #[allow(dead_code)]
    fn next_raw_key(&mut self) -> Result<Key, TermixError> {
        self.next_raw_key_timeout(Duration::new(0, 0))
    }

    pub fn next_key_timeout(&mut self, timeout: Duration) -> Result<Key, TermixError> {
        let next_key = if self.next_key.is_some() {
            self.next_key.take().unwrap()
        } else {
            self.next_raw_key_timeout(timeout)?
        };

        Ok(next_key)
    }

    fn next_raw_key_timeout(&mut self, timeout: Duration) -> Result<Key, TermixError> {
        let ch = self.next_char_timeout(timeout)?;
        match ch {
            '\u{00}' => Ok(Key::Ctrl(' ')),
            '\u{01}' => Ok(Key::Ctrl('a')),
            '\u{02}' => Ok(Key::Ctrl('b')),
            '\u{03}' => Ok(Key::Ctrl('c')),
            '\u{04}' => Ok(Key::Ctrl('d')),
            '\u{05}' => Ok(Key::Ctrl('e')),
            '\u{06}' => Ok(Key::Ctrl('f')),
            '\u{07}' => Ok(Key::Ctrl('g')),
            '\u{08}' => Ok(Key::Ctrl('h')),
            '\t' => Ok(Key::Tab),
            '\u{0a}' => Ok(Key::Ctrl('j')),
            '\u{0b}' => Ok(Key::Ctrl('k')),
            '\u{0c}' => Ok(Key::Ctrl('l')),
            '\u{0d}' => Ok(Key::Enter),
            '\u{0e}' => Ok(Key::Ctrl('n')),
            '\u{0f}' => Ok(Key::Ctrl('o')),
            '\u{10}' => Ok(Key::Ctrl('p')),
            '\u{11}' => Ok(Key::Ctrl('q')),
            '\u{12}' => Ok(Key::Ctrl('r')),
            '\u{13}' => Ok(Key::Ctrl('s')),
            '\u{14}' => Ok(Key::Ctrl('t')),
            '\u{15}' => Ok(Key::Ctrl('u')),
            '\u{16}' => Ok(Key::Ctrl('v')),
            '\u{17}' => Ok(Key::Ctrl('w')),
            '\u{18}' => Ok(Key::Ctrl('x')),
            '\u{19}' => Ok(Key::Ctrl('y')),
            '\u{1a}' => Ok(Key::Ctrl('z')),
            '\u{1b}' => self.escape_sequence(),
            '\u{7F}' => Ok(Key::Backspace),
            _ => Ok(Key::Char(ch)),
        }
    }

    fn escape_sequence(&mut self) -> Result<Key, TermixError> {
        let seq1 = self.next_char_timeout(KEY_WAIT).unwrap_or('\u{1B}');
        match seq1 {
            '[' => self.escape_csi(),
            'O' => self.escape_o(),
            _ => self.parse_alt(seq1),
        }
    }
    fn escape_o(&mut self) -> Result<Key, TermixError> {
        let seq2 = self.next_byte_timeout(KEY_WAIT)?;
        match seq2 {
            b'A' => Ok(Key::Up),    // kcuu1
            b'B' => Ok(Key::Down),  // kcud1
            b'C' => Ok(Key::Right), // kcuf1
            b'D' => Ok(Key::Left),  // kcub1
            b'F' => Ok(Key::End),   // kend
            b'H' => Ok(Key::Home),  // khome
            b'P' => Ok(Key::F(1)),  // kf1
            b'Q' => Ok(Key::F(2)),  // kf2
            b'R' => Ok(Key::F(3)),  // kf3
            b'S' => Ok(Key::F(4)),  // kf4
            b'a' => Ok(Key::CtrlUp),
            b'b' => Ok(Key::CtrlDown),
            b'c' => Ok(Key::CtrlRight), // rxvt
            b'd' => Ok(Key::CtrlLeft),  // rxvt
            _ => Err(TermixError::KeyRead(seq2)),
        }
    }
    fn escape_csi(&mut self) -> Result<Key, TermixError> {
        let seq2 = self.next_byte_timeout(KEY_WAIT)?;
        match seq2 {
            b'0' | b'9' => Err(TermixError::KeyRead(seq2)),
            b'1'..=b'8' => self.extended_escape(seq2),
            b'[' => {
                // Linux Console ESC [ [ _
                let seq3 = self.next_byte_timeout(KEY_WAIT)?;
                match seq3 {
                    b'A' => Ok(Key::F(1)),
                    b'B' => Ok(Key::F(2)),
                    b'C' => Ok(Key::F(3)),
                    b'D' => Ok(Key::F(4)),
                    b'E' => Ok(Key::F(5)),
                    _ => Err(TermixError::KeyRead(seq2)),
                }
            }
            b'A' => Ok(Key::Up),    // kcuu1
            b'B' => Ok(Key::Down),  // kcud1
            b'C' => Ok(Key::Right), // kcuf1
            b'D' => Ok(Key::Left),  // kcub1
            b'H' => Ok(Key::Home),  // khome
            b'F' => Ok(Key::End),
            b'Z' => Ok(Key::BackTab),
            b'M' => {
                unimplemented!();
            }
            b'<' => {
                unimplemented!();
            }
            _ => Err(TermixError::KeyRead(seq2)),
        }
    }
    fn extended_escape(&mut self, seq2: u8) -> Result<Key, TermixError> {
        let seq3 = self.next_byte_timeout(KEY_WAIT)?;
        if seq3 == b'~' {
            match seq2 {
                b'1' | b'7' => Ok(Key::Home), // tmux, xrvt
                b'2' => Ok(Key::Insert),
                b'3' => Ok(Key::Delete),     // kdch1
                b'4' | b'8' => Ok(Key::End), // tmux, xrvt
                b'5' => Ok(Key::PageUp),     // kpp
                b'6' => Ok(Key::PageDown),   // knp
                _ => Err(anyhow::anyhow!("todo! error handle").into()),
            }
        } else if (b'0'..=b'9').contains(&seq3) {
            let mut str_buf = String::new();
            str_buf.push(seq2 as char);
            str_buf.push(seq3 as char);

            let mut seq_last = self.next_byte_timeout(KEY_WAIT)?;
            while seq_last != b'M' && seq_last != b'~' {
                str_buf.push(seq_last as char);
                seq_last = self.next_byte_timeout(KEY_WAIT)?;
            }

            match seq_last {
                b'M' => {
                    unimplemented!()
                }
                b'~' => {
                    unimplemented!()
                }
                _ => unreachable!(),
            }
        } else if seq3 == b';' {
            let seq4 = self.next_byte_timeout(KEY_WAIT)?;
            if (b'0'..=b'9').contains(&seq4) {
                let seq5 = self.next_byte_timeout(KEY_WAIT)?;
                if seq2 == b'1' {
                    match (seq4, seq5) {
                        (b'5', b'A') => Ok(Key::CtrlUp),
                        (b'5', b'B') => Ok(Key::CtrlDown),
                        (b'5', b'C') => Ok(Key::CtrlRight),
                        (b'5', b'D') => Ok(Key::CtrlLeft),
                        (b'4', b'A') => Ok(Key::AltShiftUp),
                        (b'4', b'B') => Ok(Key::AltShiftDown),
                        (b'4', b'C') => Ok(Key::AltShiftRight),
                        (b'4', b'D') => Ok(Key::AltShiftLeft),
                        (b'3', b'H') => Ok(Key::AltHome),
                        (b'3', b'F') => Ok(Key::AltEnd),
                        (b'2', b'A') => Ok(Key::ShiftUp),
                        (b'2', b'B') => Ok(Key::ShiftDown),
                        (b'2', b'C') => Ok(Key::ShiftRight),
                        (b'2', b'D') => Ok(Key::ShiftLeft),
                        _ => Err(TermixError::KeyRead(seq2)),
                    }
                } else {
                    Err(TermixError::KeyRead(seq2))
                }
            } else {
                Err(TermixError::KeyRead(seq2))
            }
        } else {
            match (seq2, seq3) {
                (b'5', b'A') => Ok(Key::CtrlUp),
                (b'5', b'B') => Ok(Key::CtrlDown),
                (b'5', b'C') => Ok(Key::CtrlRight),
                (b'5', b'D') => Ok(Key::CtrlLeft),
                _ => Err(TermixError::KeyRead(seq2)),
            }
        }
    }
    fn parse_alt(&mut self, ch: char) -> Result<Key, TermixError> {
        match ch {
            '\u{1B}' => {
                match self.next_byte_timeout(KEY_WAIT) {
                    Ok(b'[') => {}
                    Ok(c) => {
                        return Err(TermixError::KeyRead(c));
                    }
                    Err(_) => return Ok(Key::ESC),
                }

                match self.escape_csi() {
                    Ok(Key::Up) => Ok(Key::AltUp),
                    Ok(Key::Down) => Ok(Key::AltDown),
                    Ok(Key::Left) => Ok(Key::AltLeft),
                    Ok(Key::Right) => Ok(Key::AltRight),
                    Ok(Key::PageUp) => Ok(Key::AltPageUp),
                    Ok(Key::PageDown) => Ok(Key::AltPageDown),
                    _ => Err(anyhow::anyhow!("todo: erorr handle").into()),
                }
            }
            '\u{00}' => Ok(Key::CtrlAlt(' ')),
            '\u{01}' => Ok(Key::CtrlAlt('a')),
            '\u{02}' => Ok(Key::CtrlAlt('b')),
            '\u{03}' => Ok(Key::CtrlAlt('c')),
            '\u{04}' => Ok(Key::CtrlAlt('d')),
            '\u{05}' => Ok(Key::CtrlAlt('e')),
            '\u{06}' => Ok(Key::CtrlAlt('f')),
            '\u{07}' => Ok(Key::CtrlAlt('g')),
            '\u{08}' => Ok(Key::CtrlAlt('h')),
            '\u{09}' => Ok(Key::AltTab),
            '\u{0A}' => Ok(Key::CtrlAlt('j')),
            '\u{0B}' => Ok(Key::CtrlAlt('k')),
            '\u{0C}' => Ok(Key::CtrlAlt('l')),
            '\u{0D}' => Ok(Key::AltEnter),
            '\u{0E}' => Ok(Key::CtrlAlt('n')),
            '\u{0F}' => Ok(Key::CtrlAlt('o')),
            '\u{10}' => Ok(Key::CtrlAlt('p')),
            '\u{11}' => Ok(Key::CtrlAlt('q')),
            '\u{12}' => Ok(Key::CtrlAlt('r')),
            '\u{13}' => Ok(Key::CtrlAlt('s')),
            '\u{14}' => Ok(Key::CtrlAlt('t')),
            '\u{15}' => Ok(Key::CtrlAlt('u')),
            '\u{16}' => Ok(Key::CtrlAlt('v')),
            '\u{17}' => Ok(Key::CtrlAlt('w')),
            '\u{18}' => Ok(Key::CtrlAlt('x')),
            '\u{19}' => Ok(Key::AltBackTab),
            '\u{1A}' => Ok(Key::CtrlAlt('z')),
            '\u{7F}' => Ok(Key::AltBackspace),
            ch => Ok(Key::Alt(ch)),
        }
    }

    #[allow(dead_code)]
    pub fn next_char(&mut self) -> Result<char, TermixError> {
        self.next_char_timeout(Duration::new(0, 0))
    }

    fn next_byte_timeout(&mut self, timeout: Duration) -> Result<u8, TermixError> {
        if self.byte_buf.is_empty() {
            self.fetch_bytes(timeout)?;
        }

        Ok(self.byte_buf.remove(0))
    }
    #[allow(dead_code)]
    fn next_byte(&mut self) -> Result<u8, TermixError> {
        self.next_byte_timeout(Duration::new(0, 0))
    }

    fn next_char_timeout(&mut self, timeout: Duration) -> Result<char, TermixError> {
        if self.byte_buf.is_empty() {
            self.fetch_bytes(timeout)?;
        }

        let bytes = std::mem::take(&mut self.byte_buf);
        match String::from_utf8(bytes) {
            Ok(string) => {
                let ret = string
                    .chars()
                    .next()
                    .expect("failed to get next char from input");
                self.byte_buf
                    .extend_from_slice(&string.as_bytes()[ret.len_utf8()..]);
                Ok(ret)
            }
            Err(error) => {
                let valid_up_to = error.utf8_error().valid_up_to();
                let bytes = error.into_bytes();
                let string = String::from_utf8_lossy(&bytes[..valid_up_to]);
                let ret = string
                    .chars()
                    .next()
                    .expect("failed to get next char from input");
                self.byte_buf.extend_from_slice(&bytes[ret.len_utf8()..]);
                Ok(ret)
            }
        }
    }

    fn fetch_bytes(&mut self, timeout: Duration) -> Result<(), TermixError> {
        let mut reader_buf = [0; 1];

        // clear interrupt signal
        while self.sig_rx.read(&mut reader_buf).is_ok() {}

        wait_until_ready(
            self.file.as_raw_fd(),
            Some(self.sig_rx.as_raw_fd()),
            timeout,
        )?; // wait timeout

        self.read_unread_bytes();
        Ok(())
    }

    pub fn read_unread_bytes(&mut self) {
        let mut reader_buf = [0; 1];
        while let Ok(..) = self.file.read(&mut reader_buf) {
            self.byte_buf.push(reader_buf[0]);
        }
    }
}
