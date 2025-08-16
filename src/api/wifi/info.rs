use std::net::UdpSocket;
use std::process::Command;

/// Get WiFi IP
pub fn ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

/// Get WiFi SSID on Windows
#[cfg(target_os = "windows")]
pub fn ssid() -> Option<String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .ok()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let ssid = output_str
            .lines()
            .find(|line| line.contains("SSID"))?
            .split(":")
            .nth(1)?
            .trim()
            .to_string();
        Some(ssid)
    } else {
        None
    }
}

/// Get WiFi SSID on macOS
#[cfg(target_os = "macos")]
pub fn ssid() -> Option<String> {
    let output = Command::new("networksetup")
        .args(&["-getairportnetwork", "en0"])
        .output()
        .ok()?;

    if output.status.success() {
        let ssid = String::from_utf8_lossy(&output.stdout)
            .trim()
            .split(": ")
            .nth(1)?
            .to_string();
        Some(ssid)
    } else {
        None
    }
}

/// Get WiFi SSID on Linux
#[cfg(target_os = "linux")]
pub fn ssid() -> Option<String> {
    let output = Command::new("iwgetid").arg("-r").output().ok()?;

    if output.status.success() {
        let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(ssid)
    } else {
        None
    }
}

/// Get WiFi SSID on Android
#[cfg(target_os = "android")]
pub fn ssid() -> Option<String> {
    // 使用 JNI 调用 Java 代码来获取 SSID
    // 这里需要编写相应的 Java 代码并通过 JNI 调用
    None
}

/// Get WiFi SSID on iOS
#[cfg(target_os = "ios")]
pub fn ssid() -> Option<String> {
    // 使用 FFI 调用 Objective-C 代码来获取 SSID
    // 这里需要编写相应的 Objective-C 代码并通过 FFI 调用
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_wifi_ip() {
        let ip = ip().unwrap();
        println!("{}", ip);
    }

    #[test]
    fn test_get_wifi_ssid() {
        let s = ssid().unwrap();
        println!("{}", s);
    }
}
