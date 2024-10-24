use windows::Win32::UI::WindowsAndMessaging;

pub fn reset() -> crate::Result<()> {
    unsafe {
        WindowsAndMessaging::SystemParametersInfoA(
            WindowsAndMessaging::SPI_SETDESKWALLPAPER,
            0,
            None,
            WindowsAndMessaging::SPIF_SENDCHANGE,
        )
        .unwrap();
    }

    Ok(())
}
