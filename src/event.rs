use std::fmt::Debug;

use crate::key::Key;

/// Init event and quit event is already defined.
#[derive(Clone, Debug)]
pub enum Event<CustomEvent>
where
    CustomEvent: Send + Debug,
{
    Init,
    Quit,
    Keyboard(Key),
    Custom(CustomEvent),
}
