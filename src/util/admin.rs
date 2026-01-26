pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        is_elevated::is_elevated()
    }

    #[cfg(target_os = "linux")]
    {
        unsafe { libc::geteuid() == 0 }
    }
}
