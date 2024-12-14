//! BUAA VPN API

use crate::Context;

impl Context {
    pub fn vpn_enable(&self) -> crate::Result<()> {
        let mut config = self.shared.config.write().unwrap();
        config.vpn = true;
        Ok(())
    }
    pub fn vpn_disable(&self) -> crate::Result<()> {
        let mut config = self.shared.config.write().unwrap();
        config.vpn = false;
        Ok(())
    }
}
