pub mod boya;
pub mod class;
pub mod elective;
pub mod evaluation;
pub mod office;
pub mod pan;
pub mod spoc;
mod sso;
pub mod user;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub mod wifi;

/// Marker type for BUAA SSO API Grouping, and it is the default API Group
pub struct SSO;
/// Marker type for BUAA Boya API Grouping
pub struct Boya;
/// Marker type for BUAA Smart Classroom API Grouping
pub struct Class;
pub struct Elective;
/// Marker type for BUAA Teacher Evaluation System API Grouping
pub struct Evaluation;
/// Marker type for BUAA Academic Affairs System API Grouping
pub struct AAS;
/// Marker type for BUAA Pan API Grouping
pub struct Pan;
/// Marker type for BUAA Spoc API Grouping
pub struct Spoc;
/// Marker type for BUAA User Center API Grouping
pub struct User;
/// Marker type for BUAA WiFi API Grouping
pub struct WiFi;

impl crate::Context<SSO> {
    #[inline]
    pub fn api<G>(&self) -> &crate::Context<G> {
        unsafe {
            // Safety: PhantomData 不改变实际内存布局
            &*(self as *const crate::Context<SSO> as *const crate::Context<G>)
        }
    }
    pub fn boya(&self) -> &crate::Context<Boya> {
        self.api::<Boya>()
    }
    pub fn class(&self) -> &crate::Context<Class> {
        self.api::<Class>()
    }
    pub fn elective(&self) -> &crate::Context<Elective> {
        self.api::<Elective>()
    }
    pub fn evaluation(&self) -> &crate::Context<Evaluation> {
        self.api::<Evaluation>()
    }
    pub fn office(&self) -> &crate::Context<AAS> {
        self.api::<AAS>()
    }
    pub fn pan(&self) -> &crate::Context<Pan> {
        self.api::<Pan>()
    }
    pub fn spoc(&self) -> &crate::Context<Spoc> {
        self.api::<Spoc>()
    }
    pub fn user(&self) -> &crate::Context<User> {
        self.api::<User>()
    }
    pub fn wifi(&self) -> &crate::Context<WiFi> {
        self.api::<WiFi>()
    }
}
