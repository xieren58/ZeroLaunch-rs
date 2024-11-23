use chrono::{Local, NaiveDate};
use std::fs;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use tracing::{debug, error, info, trace, warn};
use windows::core::PWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::SHGetFolderPathW;
use windows::Win32::UI::Shell::CSIDL_COMMON_STARTMENU;
use windows::Win32::UI::Shell::CSIDL_STARTMENU;
use windows::Win32::UI::Shell::KF_FLAG_DEFAULT;
use windows::Win32::UI::Shell::{FOLDERID_RoamingAppData, SHGetKnownFolderPath};
pub fn read_or_create(path: &str, content: Option<String>) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                if let Some(parent) = Path::new(path).parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        return Err(format!("无法创建文件夹: {}", e));
                    }
                }
                let initial_content = content.unwrap_or("".to_string());
                match fs::write(path, initial_content.clone()) {
                    Ok(_) => Ok(initial_content),
                    Err(write_err) => Err(format!("无法写入文件: {}", write_err)),
                }
            } else {
                Err(format!("无法读取： {}", e))
            }
        }
    }
}

/// 获取公共和用户的开始菜单路径
pub fn get_start_menu_paths() -> Result<(String, String), String> {
    // 创建缓冲区，足够存储路径
    const MAX_PATH_LEN: usize = 260;
    let mut common_path_buffer: [u16; MAX_PATH_LEN] = [0; MAX_PATH_LEN];
    let mut user_path_buffer: [u16; MAX_PATH_LEN] = [0; MAX_PATH_LEN];

    unsafe {
        // 获取公共开始菜单路径
        let hr_common = SHGetFolderPathW(
            HWND(std::ptr::null_mut()),
            CSIDL_COMMON_STARTMENU as i32,
            None,
            0,
            &mut common_path_buffer,
        );

        if hr_common.is_err() {
            return Err(format!(
                "Failed to get CSIDL_COMMON_STARTMENU: {:?}",
                hr_common
            ));
        }

        // 获取用户开始菜单路径
        let hr_user = SHGetFolderPathW(
            HWND(std::ptr::null_mut()),
            CSIDL_STARTMENU as i32,
            None,
            0,
            &mut user_path_buffer,
        );

        if hr_user.is_err() {
            return Err(format!("Failed to get CSIDL_STARTMENU: {:?}", hr_user));
        }

        // 将宽字符缓冲区转换为 Rust String
        let common_path = widestring::U16CStr::from_ptr_str(&common_path_buffer as *const u16)
            .to_string()
            .map_err(|e| format!("Failed to convert common path to string: {:?}", e))?;

        let user_path = widestring::U16CStr::from_ptr_str(&user_path_buffer as *const u16)
            .to_string()
            .map_err(|e| format!("Failed to convert user path to string: {:?}", e))?;

        debug!("自动生成路径： {common_path}, {user_path}");
        Ok((common_path, user_path))
    }
}

pub fn get_data_dir_path() -> String {
    unsafe {
        // 获取 AppData 目录
        let path = SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT.into(), None);

        // 将 PWSTR 转换为 Rust 字符串
        let path_str = path.unwrap().to_string().unwrap();
        let app_data_str = Path::new(&path_str)
            .join("ZeroLaunch-rs")
            .to_str()
            .unwrap()
            .to_string();
        info!("AppData Directory: {}", app_data_str);
        app_data_str
    }
}

/// 将一个字符串转成windows的宽字符
pub fn get_u16_vec<P: AsRef<Path>>(path: P) -> Vec<u16> {
    path.as_ref()
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// 生成当前日期的函数
pub fn generate_current_date() -> String {
    let current_date = Local::now().date_naive();
    current_date.format("%Y-%m-%d").to_string()
}

/// 比较日期字符串与当前日期的函数
pub fn is_date_current(date_str: &str) -> bool {
    // 解析输入的日期字符串
    let input_date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return false, // 如果解析失败,返回false
    };

    // 获取当前日期
    let current_date = Local::now().date_naive();

    // 比较两个日期
    input_date == current_date
}
