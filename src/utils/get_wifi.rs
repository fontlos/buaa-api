// TODO 其他平台暂时按成功处理
#[cfg(not(target_os = "windows"))]
pub fn get_wifi() -> &str {
    "BUAA-WiFi"
}

// 因为没有其他测试平台所以只做了Windows
#[cfg(target_os = "windows")]
pub fn get_wifi() -> windows::core::Result<String> {
    use windows::{
        Win32::NetworkManagement::WiFi::{
            WlanOpenHandle, WlanEnumInterfaces, WlanQueryInterface, WlanCloseHandle, WlanFreeMemory,
            WLAN_OPCODE_VALUE_TYPE, WLAN_INTERFACE_INFO_LIST, WLAN_CONNECTION_ATTRIBUTES,
            wlan_intf_opcode_current_connection
        },
        Win32::Foundation::{HANDLE, ERROR_SUCCESS}
    };
    unsafe {
        let mut client_handle: HANDLE = HANDLE::default();
        let mut negotiated_version: u32 = 0;
        let result = WlanOpenHandle(2, Some(std::ptr::null()), &mut negotiated_version, &mut client_handle);
        if result != ERROR_SUCCESS.0 {
            return Ok(format!("WlanOpenHandle failed with error: {}", result));
        }

        let mut p_if_list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let result = WlanEnumInterfaces(client_handle, Some(std::ptr::null()), &mut p_if_list);
        if result != ERROR_SUCCESS.0 {
            WlanCloseHandle(client_handle, Some(std::ptr::null()));
            return Ok(format!("WlanEnumInterfaces failed with error: {}", result));
        }

        let if_list = &*p_if_list;
        let mut ssid = String::from("Unknown Error");

        for i in 0..if_list.dwNumberOfItems {
            let p_if_info = &if_list.InterfaceInfo[i as usize];
            let mut p_conn_attr: *mut WLAN_CONNECTION_ATTRIBUTES = std::ptr::null_mut();
            let mut conn_attr_size: u32 = 0;
            let mut op_code: WLAN_OPCODE_VALUE_TYPE = WLAN_OPCODE_VALUE_TYPE(0);

            let result = WlanQueryInterface(
                client_handle,
                &p_if_info.InterfaceGuid,
                wlan_intf_opcode_current_connection,
                Some(std::ptr::null()),
                &mut conn_attr_size,
                &mut p_conn_attr as *mut _ as *mut _,
                Some(&mut op_code),
            );

            if result == ERROR_SUCCESS.0 && !p_conn_attr.is_null() {
                let conn_attr = &*p_conn_attr;
                ssid = String::from_utf8_lossy(&conn_attr.wlanAssociationAttributes.dot11Ssid.ucSSID[..conn_attr.wlanAssociationAttributes.dot11Ssid.uSSIDLength as usize]).to_string();
                WlanFreeMemory(p_conn_attr as *mut _);
                break;
            }
        }

        WlanFreeMemory(p_if_list as *mut _);
        WlanCloseHandle(client_handle, Some(std::ptr::null()));
        return Ok(ssid)
    }
}

#[test]
fn test_get_wifi() {
    let s = get_wifi().unwrap();
    println!("{}",s)
}