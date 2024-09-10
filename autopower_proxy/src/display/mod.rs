mod refresh_rate_mode;

pub use refresh_rate_mode::RefreshRateMode;

use crate::Result;
use autopower_shared::logging::Logger;
use windows::{
    core::PCWSTR,
    Win32::Graphics::Gdi::{
        ChangeDisplaySettingsW, EnumDisplayDevicesW, EnumDisplaySettingsW, CDS_TYPE, DEVMODEW,
        DISPLAY_DEVICEW, DISPLAY_DEVICE_PRIMARY_DEVICE, DISP_CHANGE_BADDUALVIEW,
        DISP_CHANGE_BADFLAGS, DISP_CHANGE_BADMODE, DISP_CHANGE_BADPARAM, DISP_CHANGE_FAILED,
        DISP_CHANGE_NOTUPDATED, DISP_CHANGE_RESTART, DISP_CHANGE_SUCCESSFUL, ENUM_CURRENT_SETTINGS,
        ENUM_DISPLAY_SETTINGS_MODE,
    },
};

const LOGGER: Logger = Logger::new("display", "autopower_proxy");

fn get_primary_display_adapter() -> Result<DISPLAY_DEVICEW> {
    let mut display_adapter = DISPLAY_DEVICEW::default();
    display_adapter.cb = size_of::<DISPLAY_DEVICEW>() as u32;

    for i in 0.. {
        unsafe {
            if !EnumDisplayDevicesW(None, i, &mut display_adapter, 0).as_bool() {
                break;
            }
        }

        if (display_adapter.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) == 0 {
            continue;
        }

        let display_adapter_string = PCWSTR::from_raw(display_adapter.DeviceString.as_ptr());
        let display_adapter_name = PCWSTR::from_raw(display_adapter.DeviceName.as_ptr());
        unsafe {
            LOGGER.debug(format!(
                "Got display adapter: {} | {}",
                display_adapter_string.display(),
                display_adapter_name.display()
            ));
        }
        return Ok(display_adapter);
    }
    Err("Could not get primary display adapter!".into())
}

fn get_current_display_mode(monitor_name: PCWSTR) -> Result<DEVMODEW> {
    LOGGER.debug(format!("Getting current display mode for {}", unsafe {
        monitor_name.display()
    }));
    let mut devmode = DEVMODEW::default();
    devmode.dmSize = size_of::<DEVMODEW>() as u16;
    unsafe {
        EnumDisplaySettingsW(Some(&monitor_name), ENUM_CURRENT_SETTINGS, &mut devmode).ok()?;
    }
    Ok(devmode)
}

fn get_display_modes_with_current_res_color() -> Result<(Vec<DEVMODEW>, DEVMODEW)> {
    LOGGER.debug("Getting all display modes with current resolution and color...");

    let monitor = get_primary_display_adapter()?;
    let monitor_name = PCWSTR::from_raw(monitor.DeviceName.as_ptr());
    let current_mode = get_current_display_mode(monitor_name)?;

    let mut devmode = DEVMODEW::default();
    devmode.dmSize = size_of::<DEVMODEW>() as u16;

    let mut buf = vec![];
    for i in 0.. {
        unsafe {
            if !EnumDisplaySettingsW(
                Some(&monitor_name),
                ENUM_DISPLAY_SETTINGS_MODE(i),
                &mut devmode,
            )
            .as_bool()
            {
                break;
            }
        }
        if devmode.dmBitsPerPel != current_mode.dmBitsPerPel
            || devmode.dmPelsHeight != current_mode.dmPelsHeight
            || devmode.dmPelsWidth != current_mode.dmPelsWidth
        {
            continue;
        }
        buf.push(devmode);
    }
    LOGGER.debug("Getting all display modes with current resolution and color...");
    Ok((buf, current_mode))
}

fn get_closest_match_display_mode(mode: RefreshRateMode) -> Result<DEVMODEW> {
    LOGGER.debug(format!(
        "Getting closest match display mode with specified refresh rate: {:?}...",
        mode
    ));

    let (refresh_rate_modes, current_mode) = get_display_modes_with_current_res_color()?;
    match mode {
        RefreshRateMode::Max => {
            let mut max = current_mode;
            for elem in &refresh_rate_modes {
                let elem_refresh = elem.dmDisplayFrequency;
                if elem_refresh > max.dmDisplayFrequency {
                    max = *elem;
                }
            }
            Ok(max)
        }
        RefreshRateMode::Value(val) => {
            let mut closest_match = current_mode;
            let mut closest_match_dist = 1000;
            for elem in &refresh_rate_modes {
                let elem_refresh = elem.dmDisplayFrequency;
                let dist = val.abs_diff(elem_refresh);
                if dist < closest_match_dist {
                    closest_match = *elem;
                    closest_match_dist = dist;
                }
            }
            Ok(closest_match)
        }
        RefreshRateMode::Min => {
            let mut min = current_mode;
            for elem in &refresh_rate_modes {
                let elem_refresh = elem.dmDisplayFrequency;
                if elem_refresh < min.dmDisplayFrequency {
                    min = *elem;
                }
            }
            Ok(min)
        }
    }
}

pub fn set_display_refresh_rate(mode: RefreshRateMode) -> Result<()> {
    LOGGER.debug(format!("Setting display refresh rate to {:?}...", mode));
    let new_mode = get_closest_match_display_mode(mode)?;
    unsafe {
        let flags = ChangeDisplaySettingsW(Some(&new_mode), CDS_TYPE(0));
        if flags != DISP_CHANGE_SUCCESSFUL {
            let msg = match flags {
                DISP_CHANGE_BADDUALVIEW => "Could not change display settings! (BADDUALVIEW)",
                DISP_CHANGE_BADFLAGS => "Could not change display settings! (BADFLAGS)",
                DISP_CHANGE_BADMODE => "Could not change display settings! (BADMODE)",
                DISP_CHANGE_BADPARAM => "Could not change display settings! (BADPARAM)",
                DISP_CHANGE_FAILED => "Could not change display settings! (FAILED)",
                DISP_CHANGE_NOTUPDATED => "Could not change display settings! (NOTUPDATED)",
                DISP_CHANGE_RESTART => "Could not change display settings! (RESTART)",
                _ => "Could not change display settings! (unknown code)",
            };
            LOGGER.error(msg);
            return Err(msg.into());
        }
    }
    Ok(())
}
