//! Defines all the keys `termix` recognizes.

// ref: https://github.com/lotabout/tuikit/blob/master/src/input.rs

#[derive(Debug, Clone)]
pub enum Key {
    Null,
    ESC,
    Ctrl(char),
    Tab,   // Ctrl-I
    Enter, // Ctrl-M

    BackTab,
    Backspace,
    AltBackTab,

    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDown,
    CtrlUp,
    CtrlDown,
    CtrlLeft,
    CtrlRight,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    AltUp,
    AltDown,
    AltLeft,
    AltRight,
    AltHome,
    AltEnd,
    AltPageUp,
    AltPageDown,
    AltShiftUp,
    AltShiftDown,
    AltShiftLeft,
    AltShiftRight,

    F(u8),

    CtrlAlt(char), // chars are lower case
    AltEnter,
    AltBackspace,
    AltTab,
    Alt(char),  // chars could be lower or upper case
    Char(char), // chars could be lower or upper case

    BracketedPasteStart,
    BracketedPasteEnd,
}
