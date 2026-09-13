use std::net::{TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

/// Check whether the current network can reach `BUAA-WiFi` or `BUAA-Mobile` gateway.
///
/// Returns `true` if a TCP connection to `gw.buaa.edu.cn:80` succeeds.
pub fn is_campus_network_reachable() -> bool {
    // 尝试连接到网关地址, 如果能连接, 则是校园网环境, 即使未登录, 但那就不是我们负责的内容了
    let timeout = Duration::from_millis(500);
    match "gw.buaa.edu.cn:80".to_socket_addrs() {
        Ok(mut addrs) => addrs.any(|addr| TcpStream::connect_timeout(&addr, timeout).is_ok()),
        Err(_) => false,
    }
}

/// Get WiFi IP
pub fn ip() -> Option<String> {
    // 发送一个虚拟包到获取本地绑定的 IP 地址
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    socket
        .local_addr()
        .ok()
        .map(|a| a.ip())
        .filter(|ip| !ip.is_loopback())
        .map(|ip| ip.to_string())
}
