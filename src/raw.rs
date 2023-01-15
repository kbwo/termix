// ref: https://github.com/lotabout/tuikit/blob/master/src/input.rs
// There are some changes.

use std::fs::File;
use std::io::{self, Write};
use std::ops;

use nix::sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg, Termios};
use nix::unistd::isatty;
use std::os::unix::io::{AsRawFd, RawFd};

use crate::error::TermixError;

pub fn get_tty() -> Box<File> {
    let tty_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .expect("Cannot get tty");
    Box::new(tty_file)
}

/// A terminal restorer, which keeps the previous state of the terminal, and restores it, when
/// dropped.
///
/// Restoring will entirely bring back the old TTY state.
pub struct RawTerminal<W: Write + AsRawFd> {
    prev_ios: Termios,
    output: W,
}

impl<W: Write + AsRawFd> RawTerminal<W> {
    pub fn hide_cursor(&mut self) -> Result<(), TermixError> {
        self.output
            .write_all(b"\x1b[?25l")
            .map_err(|_| TermixError::Write(String::from("Clearing stdout")))
    }
    pub fn show_cursor(&mut self) -> Result<(), TermixError> {
        self.output
            .write_all(b"\x1b[?25h")
            .map_err(|_| TermixError::Write(String::from("Clearing stdout")))
    }
    pub fn finish_raw(&mut self) -> Result<(), TermixError> {
        // show cursor
        self.show_cursor()?;
        let _ = tcsetattr(self.output.as_raw_fd(), SetArg::TCSANOW, &self.prev_ios);
        Ok(())
    }
}

impl<W: Write + AsRawFd> Drop for RawTerminal<W> {
    fn drop(&mut self) {
        self.finish_raw().unwrap();
    }
}

impl<W: Write + AsRawFd> ops::Deref for RawTerminal<W> {
    type Target = W;

    fn deref(&self) -> &W {
        &self.output
    }
}

impl<W: Write + AsRawFd> ops::DerefMut for RawTerminal<W> {
    fn deref_mut(&mut self) -> &mut W {
        &mut self.output
    }
}

impl<W: Write + AsRawFd> Write for RawTerminal<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

impl<W: Write + AsRawFd> AsRawFd for RawTerminal<W> {
    fn as_raw_fd(&self) -> RawFd {
        self.output.as_raw_fd()
    }
}

/// Types which can be converted into "raw mode".
///
/// # Why is this type defined on writers and not readers?
///
/// TTYs has their state controlled by the writer, not the reader. You use the writer to clear the
/// screen, move the cursor and so on, so naturally you use the writer to change the mode as well.
pub trait IntoRawMode: Write + AsRawFd + Sized {
    /// Switch to raw mode.
    ///
    /// Raw mode means that stdin won't be printed (it will instead have to be written manually by
    /// the program). Furthermore, the input isn't canonicalised or buffered (that is, you can
    /// read from stdin one byte of a time). The output is neither modified in any way.
    fn into_raw_mode(self) -> io::Result<RawTerminal<Self>>;
}

impl<W: Write + AsRawFd> IntoRawMode for W {
    // modified after https://github.com/kkawakam/rustyline/blob/master/src/tty/unix.rs#L668
    // refer: https://linux.die.net/man/3/termios
    fn into_raw_mode(self) -> io::Result<RawTerminal<W>> {
        use nix::errno::Errno::ENOTTY;
        use nix::sys::termios::OutputFlags;

        let istty = isatty(self.as_raw_fd()).map_err(nix_err_to_io_err)?;
        if !istty {
            Err(nix_err_to_io_err(ENOTTY))?
        }

        let prev_ios = tcgetattr(self.as_raw_fd()).map_err(nix_err_to_io_err)?;
        let mut ios = prev_ios.clone();
        // set raw mode
        cfmakeraw(&mut ios);
        // enable output processing (so that '\n' will issue carriage return)
        ios.output_flags |= OutputFlags::OPOST;

        tcsetattr(self.as_raw_fd(), SetArg::TCSANOW, &ios).map_err(nix_err_to_io_err)?;

        Ok(RawTerminal {
            prev_ios,
            output: self,
        })
    }
}

fn nix_err_to_io_err(err: nix::Error) -> io::Error {
    io::Error::from(err)
}
