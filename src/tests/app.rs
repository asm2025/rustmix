use rustmix::AppInfo;

pub fn test_app_info() {
    let app_info = AppInfo::lib_info();
    println!("App Info: {}", app_info);
}
