//! BUAA API

pub mod aas;
pub mod boya;
pub mod class;
pub mod cloud;
pub mod spoc;
pub mod srs;
pub mod sso;
pub mod tes;
pub mod user;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub mod wifi;

/// Marker type for Core Context, and it is the default API Group
pub struct Core;
/// Marker type for BUAA Academic Affairs System API Group
pub struct Aas;
/// Marker type for BUAA Boya Course API Group
pub struct Boya;
/// Marker type for BUAA Smart Classroom API Group
pub struct Class;
/// Marker type for BUAA Cloud Disk API Group
pub struct Cloud;
/// Marker type for BUAA Spoc Platform API Group
pub struct Spoc;
/// Marker type for BUAA Undergraduate & Graduate Student Course Registration System API Group
pub struct Srs;
/// Marker type for BUAA SSO API Group
pub struct Sso;
/// Marker type for BUAA Teacher Evaluation System API Group
pub struct Tes;
/// Marker type for BUAA User Center API Group
pub struct User;
/// Marker type for BUAA WiFi API Group
pub struct Wifi;

// 这必须和 Marker 一一对应, 因为 Token trait 使用宏进行转化
/// Api location marker
#[derive(Debug, Eq, PartialEq)]
pub enum Location {
    /// BUAA Academic Affairs System API
    Aas,
    /// BUAA Boya Course API
    Boya,
    /// BUAA Smart Classroom API
    Class,
    /// BUAA Cloud Disk API
    Cloud,
    /// BUAA Spoc Platform API
    Spoc,
    /// BUAA Undergraduate & Graduate Student Course Registration System API
    Srs,
    /// BUAA SSO API
    Sso,
    /// BUAA Teacher Evaluation System API
    Tes,
    /// BUAA User Center API
    User,
    /// BUAA WiFi API
    Wifi,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl crate::Context<Core> {
    /// Get BUAA Academic Affairs System API Group
    pub const fn aas(&self) -> &crate::Context<Aas> {
        self.api::<Aas>()
    }
    /// Get BUAA Boya Course API Group
    pub const fn boya(&self) -> &crate::Context<Boya> {
        self.api::<Boya>()
    }
    /// Get BUAA Smart Classroom API Group
    pub const fn class(&self) -> &crate::Context<Class> {
        self.api::<Class>()
    }
    /// Get BUAA Cloud Disk API Group
    pub const fn cloud(&self) -> &crate::Context<Cloud> {
        self.api::<Cloud>()
    }
    /// Get BUAA Spoc Platform API Group
    pub const fn spoc(&self) -> &crate::Context<Spoc> {
        self.api::<Spoc>()
    }
    /// Get BUAA User Center API Group
    pub const fn user(&self) -> &crate::Context<User> {
        self.api::<User>()
    }
    /// Get BUAA Undergraduate & Graduate Student Course Registration System API Group
    pub const fn srs(&self) -> &crate::Context<Srs> {
        self.api::<Srs>()
    }
    /// Get BUAA SSO API Group
    pub const fn sso(&self) -> &crate::Context<Sso> {
        self.api::<Sso>()
    }
    /// Get BUAA Teacher Evaluation System API Group
    pub const fn tes(&self) -> &crate::Context<Tes> {
        self.api::<Tes>()
    }
    /// Get BUAA WiFi API Group
    pub const fn wifi(&self) -> &crate::Context<Wifi> {
        self.api::<Wifi>()
    }
}
