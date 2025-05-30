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
/// Marker type for BUAA Academic Affairs System API Grouping
pub struct Aas;
/// Marker type for BUAA App API Grouping
pub struct App;
/// Marker type for BUAA Boya Course API Grouping
pub struct Boya;
/// Marker type for BUAA Smart Classroom API Grouping
pub struct Class;
/// Marker type for BUAA Cloud Disk API Grouping
pub struct Cloud;
/// Marker type for BUAA Spoc Platform API Grouping
pub struct Spoc;
/// Marker type for BUAA Undergraduate & Graduate Student Course Registration System API Grouping
pub struct Srs;
/// Marker type for BUAA SSO API Grouping
pub struct SSO;
/// Marker type for BUAA Teacher Evaluation System API Grouping
pub struct Tes;
/// Marker type for BUAA User Center API Grouping
pub struct User;
/// Marker type for BUAA WiFi API Grouping
pub struct WiFi;

impl<G> crate::Context<G> {
    /// Obtains a type-state view for the specified API group
    ///
    /// This zero-cost conversion provides access to group-specific APIs
    /// while sharing the same underlying context.
    ///
    /// # Safety
    /// The cast is safe because:
    /// 1. `PhantomData<G>` has no runtime representation
    /// 2. All context data is stored in `Arc`-wrapped fields
    /// 3. The original context remains accessible
    #[inline]
    pub fn api<N>(&self) -> &crate::Context<N> {
        unsafe {
            // Safety: PhantomData 不改变实际内存布局
            &*(self as *const crate::Context<G> as *const crate::Context<N>)
        }
    }
}

impl crate::Context<Core> {
    /// Get BUAA Academic Affairs System API Group
    pub fn aas(&self) -> &crate::Context<Aas> {
        self.api::<Aas>()
    }
    /// Get BUAA App API Group
    pub fn app(&self) -> &crate::Context<App> {
        self.api::<App>()
    }
    /// Get BUAA Boya Course API Group
    pub fn boya(&self) -> &crate::Context<Boya> {
        self.api::<Boya>()
    }
    /// Get BUAA Smart Classroom API Group
    pub fn class(&self) -> &crate::Context<Class> {
        self.api::<Class>()
    }
    /// Get BUAA Cloud Disk API Group
    pub fn cloud(&self) -> &crate::Context<Cloud> {
        self.api::<Cloud>()
    }
    /// Get BUAA Spoc Platform API Group
    pub fn spoc(&self) -> &crate::Context<Spoc> {
        self.api::<Spoc>()
    }
    /// Get BUAA User Center API Group
    pub fn user(&self) -> &crate::Context<User> {
        self.api::<User>()
    }
    /// Get BUAA Undergraduate & Graduate Student Course Registration System API Group
    pub fn srs(&self) -> &crate::Context<Srs> {
        self.api::<Srs>()
    }
    /// Get BUAA SSO API Group
    pub fn sso(&self) -> &crate::Context<SSO> {
        self.api::<SSO>()
    }
    /// Get BUAA Teacher Evaluation System API Group
    pub fn tes(&self) -> &crate::Context<Tes> {
        self.api::<Tes>()
    }
    /// Get BUAA WiFi API Group
    pub fn wifi(&self) -> &crate::Context<WiFi> {
        self.api::<WiFi>()
    }
}
