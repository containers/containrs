use crate::sandbox::{Sandbox, State};
use anyhow::{bail, Result};

pub struct PinnedSandbox {
    state: State,
}

impl Sandbox for PinnedSandbox {
    fn create() -> Result<Self> {
        Ok(PinnedSandbox {
            state: State::Created,
        })
    }

    fn start(&mut self) -> Result<()> {
        if self.state() != State::Created {
            bail!("not in created state ({})", self.state())
        }
        self.state = State::Started;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if self.state() != State::Started {
            bail!("not in started state ({})", self.state())
        }
        self.state = State::Stopped;
        Ok(())
    }

    fn remove(self) {}

    fn state(&self) -> State {
        self.state
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn pod_state_success() -> Result<()> {
        let mut p = PinnedSandbox::create()?;
        assert_eq!(p.state(), State::Created);

        p.start()?;
        assert_eq!(p.state(), State::Started);

        p.stop()?;
        assert_eq!(p.state(), State::Stopped);

        p.remove();
        Ok(())
    }

    #[test]
    fn pod_start_fail_not_in_created_state() -> Result<()> {
        let mut p = PinnedSandbox::create()?;
        p.start()?;
        p.stop()?;
        assert!(p.start().is_err());
        Ok(())
    }

    #[test]
    fn pod_stop_fail_not_in_stopped_state() -> Result<()> {
        let mut p = PinnedSandbox::create()?;
        assert!(p.stop().is_err());
        Ok(())
    }
}
