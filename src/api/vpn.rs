//! BUAA VPN API

use crate::{Session, SessionError};

impl Session {
    pub fn vpn_enable(&self) -> Result<(), SessionError> {
        let mut config = self.config.write().unwrap();
        config.vpn = true;
        Ok(())
    }
    pub fn vpn_disable(&self) -> Result<(), SessionError> {
        let mut config = self.config.write().unwrap();
        config.vpn = false;
        Ok(())
    }
}
