[EN](./Readme.md)
# BUAA API

> Make BUAA Great Again

## TodoList

- [x] 统一身份认证 => `sso_login`
- [x] BUAA WiFi 登录 => `wifi_login`
- [x] 用户中心 => `user_*`
  - [x] 登录
  - [x] 获取状态
- [ ] Spoc 平台
  - [x] 登录
  - [x] 查询课表
- [ ] 本研选课
- [x] 博雅课程 => `boya_*`
  - [x] 登录
  - [x] 查询课程
  - [x] 选课
  - [x] 退课
  - [x] 一个用于扩展的通用请求 API
- [x] 智慧教室 => `class_*`
  - [x] 登录
  - [x] 签到
  - [x] 查询
- [ ] BUAA App: 一些杂项 => `app_*`
  - [x] 登录
  - [x] 查询课表

不在上述列表中的 API, 它可能是被遗忘或者我认为不重要, 但如果你需要, 欢迎提 issue 或者提交 pr

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