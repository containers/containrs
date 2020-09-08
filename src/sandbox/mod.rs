use anyhow::Result;
use std::fmt;

mod pinned;

pub trait Sandbox {
    /// Create sets-up a new sandbox.
    fn create() -> Result<Self>
    where
        Self: Sized;

    /// Start runs a previously created sandbox.
    fn start(&mut self) -> Result<()>;

    /// Stop a previously started sandbox.
    fn stop(&mut self) -> Result<()>;

    /// Remove a stopped sandbox.
    fn remove(self);

    /// Retrieve the current state of the sandbox.
    fn state(&self) -> State;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Possible states a sandbox can have.
pub enum State {
    /// A sandbox is in `Created` state after the `create()` method has been called.
    Created,

    /// A sandbox is in `Started` state after the `start()` method has been called.
    Started,

    /// A sandbox is in `Stopped` state after the `stop()` method has been called.
    Stopped,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            State::Created => write!(f, "created"),
            State::Started => write!(f, "started"),
            State::Stopped => write!(f, "stopped"),
        }
    }
}
