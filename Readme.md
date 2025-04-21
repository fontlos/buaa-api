# BUAA API

> Make BUAA Great Again

> 注: 本项目只用于学习分享, 请于下载后 24 小时内删除, 使用产生的一切问题由使用者自行承担, 如有侵权我将删除此储存库和软件
>
> Tips: This project is only for learning and sharing, please delete within 24 hours after downloading, all problems caused by the use are borne by the user, if there is any infringement I will delete this repository and software

## TodoList

- [x] SSO Login: `login` built in `Context`
- [x] BUAA WiFi: `wifi`
  - [x] Login
  - [x] Logout
- [x] User Center: `user`
  - [x] login
  - [x] get state
- [ ] Spoc: `spoc`
  - [x] login
  - [x] query class table
- [ ] Course selection
- [x] Boya Course: `boya`
  - [x] login
  - [x] query
  - [x] select
  - [x] drop
  - [x] A universal request API for extensions
- [x] Smart Classroom: `class`
  - [x] login
  - [x] checkin
  - [x] query

APIs not listed above might have been overlooked or deemed unimportant by me, but if you need them, feel free to open an issue or submit a pull request.

# Usage

The basic process is:

- Initialize the `Context`
- Set account
- (Optional) Specifies the file used to read and write cookies
- Login to context
- Get a subdomain instance
- Login to subdomain
- Call API in subdomain

There is a simple example:

```rust
use buaa::Context;

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

    // Get a subdomain instance
    let user = context.user();
    // Login to subdomain
    user.login().await.unwrap();

    // Call API in subdomain
    let state = user.get_state().await.unwrap();
    println!("{}", state);

    // (Optional) Save cookies to file
    context.save();
}
```

A more complex example:

```rust
use buaa::Context;

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

    let id = 1; // Get from course_list
    let res = boya.select_course(6637).await.unwrap();
    println!("{}", res);

    context.save();
}
```

BUAA WiFi is independent of other APIs and does not require cookies or `context.login`, so you need to provide a separate username and password:

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