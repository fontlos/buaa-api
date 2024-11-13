[中文](./ReadmeCN.md)
# BUAA API

> Make BUAA Great Again

# Tip

The names of APIs might seem unusual because they are derived from their corresponding subdomains. For example, the naming convention for APIs under `gw.buaa.edu.cn` is `gw_*`

## Todo

- [x] SSO Login => `login`
- [x] BUAA WiFi Login => `gw_login`
- [x] User Center => `uc_*`
  - [x] login
  - [x] get state
- [ ] Spoc
- [ ] Course selection
- [ ] Boya Course => `bykc_*`
  - [x] login
  - [x] query
  - [ ] select
  - [ ] drop
- [x] Smart Classroom => `iclass_*`
  - [x] login
  - [x] checkin
  - [x] query

APIs not listed above might have been overlooked or deemed unimportant by me, but if you need them, feel free to open an issue or submit a pull request.

# Example

```rust
use buaa::Session;

#[tokio::main]
async fn main() {
    let mut session = Session::new_in_file("cookie.json");

    session.sso_login("username", "password").await.unwrap();

    session.uc_login().await.unwrap();
    let state = session.uc_get_state().await.unwrap();
    println!("{}", state);

    session.gw_login("username", "password").await.unwrap();
    session.save();
}
```