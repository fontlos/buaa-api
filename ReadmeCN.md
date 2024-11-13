[EN](./Readme.md)
# BUAA API

> Make BUAA Great Again

## Tip

API 的名字可能很奇怪, 因为它们是由对应的四级域名派生而来的, 比如 `gw.buaa.edu.cn` 下的 API 名称命名就是 `gw_*`

## TodoList

- [x] 统一身份认证 => `login`
- [x] BUAA WiFi 登录 => `gw_login`
- [x] 用户中心 => `uc_*`
  - [x] 登录
  - [x] 获取状态
- [ ] Spoc 平台
- [ ] 本研选课
- [ ] 博雅课程 => `bykc_*`
  - [x] 登录
  - [x] 查询课程
  - [ ] 选课
  - [ ] 退课
- [x] 智慧教室 => `iclass_*`
  - [x] 登录
  - [x] 签到
  - [x] 查询

不在上述列表中的 API, 它可能是被遗忘或者我认为不重要, 但如果你需要, 欢迎提 issue 或者提交 pr

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