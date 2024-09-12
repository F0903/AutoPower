# AutoPower

A small, efficient and configurable Windows service that switches your Windows laptop's power mode and refresh rate when plugged in and out of power.

Uses minimal CPU, as code only runs when you plug your laptop in or out of power. It also allocates minimal memory, so you don't need to worry about any performance impact when the service is running.

## Installation

**Note: Only Windows 10 AND ABOVE is supported due to API requirements.**  
**Note: Beware that antivirus might flag the service or install scripts.**

- Download the release.zip from the latest release.
- Copy both .exe files and all .ps1 scripts to your desired location (recommended to be a non-admin location like 'C:\autopower\').
- Run the install_service.ps1 script.
- Everything should now work!
- It is recommended to restart your PC afterwards, as this makes it work reliably.

## Configuration

After the service has started, a `config.json` file should appear in the installation directory.
Here you can change options for both the wired and battery powered configurations, such as the refresh rate, power scheme, and whether or not you want a desktop notification or if you want the service change the refresh rate at all.

You do not need to restart the service to load the new changes, as the config file is read each time the power state changes.

Beware that an invalid configuration will override the whole file with the defaults.

### Configuring power schemes

In `config.json`, you can configure the `power_scheme` field in either `wired_config` or `battery_config` with the following values:

#### High Performance
```json
"power_scheme": "HighPerformance",
```
#### Balanced
```json
"power_scheme": "Balanced",
```
[_Note: only Balanced works with modern standby._](https://learn.microsoft.com/en-us/windows/win32/power/power-policy-settings)

#### Power Saver
```json
"power_scheme": "PowerSaver",
```
#### Custom
```json
"power_scheme": {
  "Custom": "**your_custom_guid***"
},
```

## Building

It's important to use the `--workspace` switch when building so all binaries get built. (eg. `cargo build --release --workspace`)

## Uninstallation

- Open your installation directory.
- Run uninstall_service.ps1.
- After this you can manually delete the installation directory.
