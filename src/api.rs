pub mod aas;
pub mod app;
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
/// Marker type for BUAA App API Group
pub struct App;
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

// TODO: 也许可以用 Trait + 上面的标记结构体来替代下面的枚举
// ```
// pub trait ApiGroup {
//     type Field;
//     const EXPIRATION: u64;
//     fn get_field(store: &mut CredentialStore) -> &mut Self::Field;
// }
// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Location {
    Aas,
    App,
    Boya,
    Class,
    Cloud,
    Spoc,
    Srs,
    Sso,
    Tes,
    User,
    Wifi,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<G> crate::Context<G> {
    /// Obtains a type-state view for the specified API group
    ///
    /// This zero-cost conversion provides access to group-specific APIs
    /// while sharing the same underlying context.
    ///
    /// # Safety
    ///
    /// The cast is safe because:
    ///
    /// 1. `PhantomData<G>` has no runtime representation
    /// 2. All context data is stored in `Arc`-wrapped fields
    /// 3. The original context remains accessible
    #[inline]
    pub const fn api<N>(&self) -> &crate::Context<N> {
        unsafe {
            // Safety: PhantomData 不改变实际内存布局
            &*(self as *const crate::Context<G> as *const crate::Context<N>)
        }
    }
}

impl crate::Context<Core> {
    /// Get BUAA Academic Affairs System API Group
    pub const fn aas(&self) -> &crate::Context<Aas> {
        self.api::<Aas>()
    }
    /// Get BUAA App API Group
    pub const fn app(&self) -> &crate::Context<App> {
        self.api::<App>()
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
