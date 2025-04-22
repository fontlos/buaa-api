# BUAA API

> Make BUAA Great Again

> 注: 本项目只用于学习分享, 请于下载后 24 小时内删除, 使用产生的一切问题由使用者自行承担, 如有侵权我将删除此储存库和软件
>
> Tips: This project is only for learning and sharing, please delete within 24 hours after downloading, all problems caused by the use are borne by the user, if there is any infringement I will delete this repository and software

## TodoList

- [x] BUAA SSO Login: `login` built in `Context`
- [ ] BUAA Academic Affairs System
- [x] BUAA Boya Course: `boya`
  - [x] Login
  - [x] Query
  - [x] Select
  - [x] Drop
  - [x] A universal request API for extensions
- [x] BUAA Smart Classroom: `class`
  - [x] Login
  - [x] Checkin
  - [x] Query
- [ ] BUAA Cloud Disk: `cloud`
  - [x] Login
- [ ] Spoc Platform: `spoc`
  - [x] Login
  - [x] Query class schedule
- [ ] BUAA Undergraduate & Graduate Student Course Registration System
  - [x] Login
- [ ] BUAA Teacher Evaluation System
  - [x] Login
- [x] User Center: `user`
  - [x] Login
  - [x] Get state
- [x] BUAA WiFi: `wifi`
  - [x] Login
  - [x] Logout


APIs not listed above might have been overlooked or deemed unimportant by me, but if you need them, feel free to open an issue or submit a pull request.

# Usage

The basic process is:

- Initialize the `Context`
- Set account
- (Optional) Specifies the file used to read and write cookies and config
- Login to SSO (Context)
- Get the API group you need
- Login to this group
- Call API in this group

There is a simple example:

```rust
use buaa_api::Context;

#[tokio::main]
async fn main() {
    // Initialize the `Context`
    let context = Context::new();
    // Set account
    context.set_account("username", "password");
    // Specifies the file used to read and write cookies
    context.with_cookies("cookie.json");
    // Login to context
    context.login().await.unwrap();

    // Get an API Group
    let user = context.user();
    // Login to this group
    user.login().await.unwrap();

    // Call API in this group
    let state = user.get_state().await.unwrap();
    println!("{}", state);

    // (Optional) Save cookies to file
    context.save_cookie("cookie.json");
}
```

A more complex example:

```rust
use buaa_api::Context;

#[tokio::main]
async fn main() {
    let context = Context::new();
    context.set_account("username", "password")
    context.with_cookies("cookie.json");
    context.login().await.unwrap();

    let boya = context.boya();
    boya.login().await.unwrap();

    let course_list = boya.query_course().await.unwrap();
    println!("{}", course_list);

    let id = 1; // Should get from course_list, just an example
    let res = boya.select_course(id).await.unwrap();
    println!("{}", res);

    context.save();
}
```

BUAA WiFi is independent of other APIs and does not require cookies or `Context.login()`:

```rust
use buaa::Context;

#[tokio::main]
async fn main() {
    let context = Context::new();
    context.set_account("username", "password")
    let wifi = context.wifi();
    // Login to BUAA WiFi
    wifi.login().await.unwrap();
    // Logout to BUAA WiFi
    wifi.logout().await.unwrap();
}
```
