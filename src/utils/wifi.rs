pub fn get_wifi_ip() -> Option<String> {
    let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };
    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    }
    match socket.local_addr() {
        Ok(a) => Some(a.ip().to_string()),
        Err(_) => None,
    }
}

#[cfg(target_os = "windows")]
pub fn get_wifi_ssid() -> Option<String> {
    use std::process::Command;

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

#[cfg(target_os = "macos")]
pub fn get_wifi_ssid() -> Option<String> {
    use std::process::Command;

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

#[cfg(target_os = "linux")]
pub fn get_wifi_ssid() -> Option<String> {
    use std::process::Command;

    let output = Command::new("iwgetid").arg("-r").output().ok()?;

    if output.status.success() {
        let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(ssid)
    } else {
        None
    }
}

#[cfg(target_os = "android")]
pub fn get_wifi_ssid() -> Option<String> {
    // 使用 JNI 调用 Java 代码来获取 SSID
    // 这里需要编写相应的 Java 代码并通过 JNI 调用
    None
}

#[cfg(target_os = "ios")]
pub fn get_wifi_ssid() -> Option<String> {
    // 使用 FFI 调用 Objective-C 代码来获取 SSID
    // 这里需要编写相应的 Objective-C 代码并通过 FFI 调用
    None
}

#[test]
fn test_get_wifi_ssid() {
    let s = get_wifi_ssid().unwrap();
    println!("{}", s);
}
