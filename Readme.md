[中文](./ReadmeCN.md)
# BUAA API

> Make BUAA Great Again

## Todo

- [x] SSO Login => `sso_login`
- [x] BUAA WiFi Login => `wifi_login`
- [x] User Center => `user_*`
  - [x] login
  - [x] get state
- [ ] Spoc => `spoc_*`
  - [x] login
  - [x] query class table
- [ ] Course selection
- [x] Boya Course => `boya_*`
  - [x] login
  - [x] query
  - [x] select
  - [x] drop
  - [x] A universal request API for extensions
- [x] Smart Classroom => `class_*`
  - [x] login
  - [x] checkin
  - [x] query
- [ ] BUAA App => `app_*`
  - [x] login
  - [x] query class table

APIs not listed above might have been overlooked or deemed unimportant by me, but if you need them, feel free to open an issue or submit a pull request.

# Example

```rust
use buaa::Session;

#[tokio::main]
async fn main() {
    let mut session = Session::new_in_file("cookie.json");

    session.sso_login("username", "password").await.unwrap();

    session.user_login().await.unwrap();
    let state = session.user_get_state().await.unwrap();
    println!("{}", state);

    session.save();
}
```