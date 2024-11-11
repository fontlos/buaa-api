# BUAA API

> Make BUAA Great Again

> Tip: The prefix of most API names is derived from the fourth-level domain name of the corresponding domain name

## Todo

- [x] SSO Login
- [x] BUAA WiFi Login
- [ ] Spoc
- [ ] Course selection
- [ ] Boya Course
- [ ] Smart Classroom
  - [x] login
  - [ ] checkin

# Example

```rust
use buaa::Session;

#[tokio::main]
async fn main() {
    let mut session = Session::new_in_file("cookie.json");

    session.login("username", "password").await.unwrap();

    session.uc_login().await.unwrap();
    let state = session.uc_get_state().await.unwrap();
    println!("{}", state);

    session.gw_login("username", "password").await.unwrap();
    session.save();
}
```