//! BUAA User Center API

mod opt;

/// BUAA User Center API Wrapper <br>
/// Call `user()` on `Context` to get an instance of this struct and call corresponding API on this instance.
#[wrap_api::wrap_api(user)]
struct UserCenterAPI;
