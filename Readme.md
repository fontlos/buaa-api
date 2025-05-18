# BUAA API

> Make BUAA Great Again

> 注: 本项目只用于学习分享, 请于下载后 24 小时内删除, 使用产生的一切问题由使用者自行承担, 如有侵权我将删除此储存库和软件
>
> Tips: This project is only for learning and sharing, please delete within 24 hours after downloading, all problems caused by the use are borne by the user, if there is any infringement I will delete this repository and software

## TodoList

- [x] BUAA SSO Login: `login` built in `Context`
- [ ] BUAA Academic Affairs System
  - [ ] Login
- [x] BUAA Boya Course: `boya`
  - [x] Login
  - [x] Query course
  - [x] Query selected
  - [x] Query statistic information
  - [x] Select course
  - [x] Drop course
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
  - [x] A universal request API for extensions
- [x] BUAA Undergraduate & Graduate Student Course Registration System
  - [x] Login
  - [x] Get filter
  - [x] Query with filter
  - [x] Query selected
  - [x] Select course
  - [x] Drop course
- [x] BUAA Teacher Evaluation System
  - [x] Login
  - [x] Get list
  - [x] Get form
  - [x] Submit form
- [ ] User Center: `user`
  - [x] Login
  - [x] Get state
  - [ ] Change password
- [ ] BUAA WiFi: `wifi`
  - [x] Login
  - [x] Logout
  - [ ] Recharge


APIs not listed above might have been overlooked or deemed unimportant by me, but if you need them, feel free to open an issue or submit a pull request.

# Usage

The basic process is:

- Initialize the `Context`
- (Optional) Set account
- (Optional) Specifies the dictionary for auth
- (Auto default) Login to SSO (Context)
- Get the API group you need
- (Auto default) Login to this group
- Call API in this group
- (Optional) Save auth file

There are some simple examples:

```rust
use buaa_api::Context;

#[tokio::main]
async fn main() {
    // Initialize the `Context`
    let context = Context::new();
    // Set account
    context.set_account("username", "password");
    // Login to context, and it will auto refresh by default
    // context.login().await.unwrap();

    // Get an API Group
    let user = context.user();
    // Login to this group, and it will auto refresh by default
    // user.login().await.unwrap();

    // Call API in this group
    let state = user.get_state().await.unwrap();
    println!("{}", state);
}
```

```rust
use buaa_api::Context;

#[tokio::main]
async fn main() {
    let context = Context::with_auth("./data");
    context.login().await.unwrap();

    let boya = context.boya();
    boya.login().await.unwrap();

    let course_list = boya.query_course().await.unwrap();
    println!("{}", course_list);

    let id = 1; // Should get from course_list, just an example
    let res = boya.select_course(id).await.unwrap();
    println!("{}", res);

    context.save_auth();
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
