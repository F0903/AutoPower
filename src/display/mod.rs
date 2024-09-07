mod refresh_rate_mode;

pub use refresh_rate_mode::RefreshRateMode;

use crate::Result;
use autopower_shared::logging::Logger;
use windows::{
    core::PCWSTR,
    Win32::Graphics::Gdi::{
        ChangeDisplaySettingsW, EnumDisplayDevicesW, EnumDisplaySettingsW, CDS_TYPE, DEVMODEW,
        DISPLAY_DEVICEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_PRIMARY_DEVICE,
        DISP_CHANGE_BADDUALVIEW, DISP_CHANGE_BADFLAGS, DISP_CHANGE_BADMODE, DISP_CHANGE_BADPARAM,
        DISP_CHANGE_FAILED, DISP_CHANGE_NOTUPDATED, DISP_CHANGE_RESTART, DISP_CHANGE_SUCCESSFUL,
        ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_MODE,
    },
};

const LOGGER: Logger = Logger::new("display", "autopower");

fn get_primary_display_adapter() -> Result<DISPLAY_DEVICEW> {
    LOGGER.debug("Getting primary display adapter...");
    let mut display_adapter = DISPLAY_DEVICEW::default();
    display_adapter.cb = size_of::<DISPLAY_DEVICEW>() as u32;
    for i in 0.. {
        unsafe {
            if !EnumDisplayDevicesW(None, i, &mut display_adapter, 0).as_bool() {
                break;
            }

            let display_adapter_name = PCWSTR::from_raw(display_adapter.DeviceString.as_ptr());
            LOGGER.debug(format!(
                "Got display adapter: {}",
                display_adapter_name.display()
            ));

            if (display_adapter.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0 {
                return Ok(display_adapter);
            }
        }
    }

    Err("Could not get primary display adapter!".into())
}

pub fn get_primary_monitor() -> Result<(DISPLAY_DEVICEW, PCWSTR)> {
    LOGGER.debug("Getting primary monitor...");
    let display_adapter = get_primary_display_adapter()?;
    let display_adapter_name = PCWSTR::from_raw(display_adapter.DeviceName.as_ptr());
    unsafe {
        println!("{}", display_adapter_name.display());
    }
    let mut display = DISPLAY_DEVICEW::default();
    display.cb = size_of::<DISPLAY_DEVICEW>() as u32;

    for i in 0.. {
        unsafe {
            if !EnumDisplayDevicesW(Some(&display_adapter_name), i, &mut display, 0).as_bool() {
                break;
            }

            let display_name = PCWSTR::from_raw(display.DeviceName.as_ptr());
            println!("{}", display_name.display());

            if (display.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0 {
                return Ok((display, PCWSTR::from_raw(display.DeviceName.as_ptr())));
            }
        }
    }

    Err("Could not get primary monitor!".into())
}

fn get_current_display_mode() -> Result<DEVMODEW> {
    LOGGER.debug("Getting current display mode...");
    let (_, monitor_name) = get_primary_monitor()?;
    let mut devmode = DEVMODEW::default();
    devmode.dmSize = size_of::<DEVMODEW>() as u16;
    unsafe {
        EnumDisplaySettingsW(Some(&monitor_name), ENUM_CURRENT_SETTINGS, &mut devmode).ok()?;
    }
    Ok(devmode)
}

fn get_display_modes_with_current_res_color() -> Result<(Vec<DEVMODEW>, DEVMODEW)> {
    LOGGER.debug("Getting all display modes with current resolution and color...");
    let (_, monitor_name) = get_primary_monitor()?;
    let current_mode = get_current_display_mode()?;
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
