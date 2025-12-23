# BUAA API

> An all in one library. Aim To Make BUAA Great Again

> 注: 本项目只用于学习分享, 请于下载后 24 小时内删除, 使用产生的一切问题由使用者自行承担, 如有侵权我将删除此储存库和软件
>
> Tips: This project is only for learning and sharing, please delete within 24 hours after downloading, all problems caused by the use are borne by the user, if there is any infringement I will delete this repository and software

## TodoList

- [x] BUAA SSO: `sso`
  - [x] Login: built in `Context`
- [ ] BUAA Academic Affairs System: `aas`
  - [ ] Login
- [x] BUAA Boya Course: `boya`
  - [x] Login
  - [x] Query courses list
  - [x] Query single course
  - [x] Query selected courses
  - [x] Query statistic information
  - [x] Select course
  - [x] Drop course
  - [x] Checkin course
  - [x] Checkout course
  - [x] A universal request API for extensions
- [x] BUAA Smart Classroom: `class`
  - [x] Login
  - [x] Query schedule by date
  - [x] Query course
  - [x] Query course schedule
  - [x] Checkin class
- [ ] BUAA Cloud Disk: `cloud`
  - [x] Login
  - [x] Dir & File
    - [x] Get root dir
    - [x] Get user root dir
    - [x] List dir
    - [x] Get item size
    - [x] Get suggest name
  - [x] CRUD
    - [x] Create dir
    - [x] Rename item
    - [x] Move item
    - [x] Copy item
    - [x] Delete item
  - [x] Recycle Bin
    - [x] List
    - [x] Delete
    - [x] Restore
  - [x] Share
    - [x] Share record
    - [x] Share item
    - [x] Share update
    - [x] Share delete
  - [x] Download
    - [x] Get single item download URL
    - [x] Get a zip of multiple items download URL
    - [x] Auto get items download URL
  - [x] Upload item
    - [x] Check hash
    - [x] Fast upload (Check hash success)
    - [x] Get upload authorization
    - [x] Upload (Need `multipart` feature)
- [ ] Spoc Platform: `spoc`
  - [x] Login
  - [x] Query teaching week
  - [x] Query weekly schedule
  - [x] A universal request API for extensions
- [x] BUAA Undergraduate & Graduate Student Course Registration System: `srs`
  - [x] Login
  - [x] Get filter
  - [x] Query courses with filter
  - [x] Query pre-selected courses
  - [x] Query selected courses
  - [x] Pre-select course
  - [x] Select course
  - [x] Drop course
- [x] BUAA Teaching Evaluation System: `tes`
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


APIs not listed above might have been overlooked or deemed unimportant by me, but if you need them, feel free to open an issue or open a pull request.

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
    let context = Context::with_auth("./data").unwrap();

    let boya = context.boya();
    // You can manually relogin in advance.
    // This is useful to prevent login invalidation when grabbing a course
    boya.login().await.unwrap();

    let course_list = boya.query_course().await.unwrap();
    println!("{}", course_list);

    // Waiting until course can be selected ...

    let id = course_list[0].id; // Just an example
    let res = boya.select_course(id).await.unwrap();
    println!("{}", res);

    context.save_auth().unwrap();
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

More usage see [`examples`](./examples)
