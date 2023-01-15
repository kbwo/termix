//! Trait of model and useful types

use std::fmt::Debug;

use crate::event::Event;

/// Type of callback function
pub type Cmd<CustomEvent> = fn() -> Event<CustomEvent>;

/// Type of model at next render
pub type NextModel<Model, CustomEvent> = Option<Box<dyn ModelAct<Model, CustomEvent>>>;

/// Return type of update method of ModelAct trait
pub type Updater<Model, CustomEvent> = (NextModel<Model, CustomEvent>, Option<Cmd<CustomEvent>>);

/// ModelAct must be implemented to pass Program
/// One big difference with bubbletea, there isn't init method.
/// To handle model and command initialization, you must write logic in
/// `update` method.
pub trait ModelAct<Model, CustomEvent>
where
    CustomEvent: Send + Debug,
{
    /// Define logic how to handle models.
    /// Updater.0 is model with state on next render
    /// Updater.1 is callback function to fire next event
    fn update(&self, event: &Event<CustomEvent>) -> Updater<Model, CustomEvent>;
    /// Define UI
    fn view(&self) -> String;
}
