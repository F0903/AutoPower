# AutoPower

A small and efficient Windows service that automatically switches your laptop between the Balanced and High Performance power modes when you plug it in or out of power, and sends you a notification.

Uses virtually no CPU, as code only runs when you plug your laptop in or out of power. It also allocates minimal memory, so you don't need to worry about any performance impact when the service is running.

## Installation

**Note: Only Windows 11/10 is supported due to API requirements.**  
**Note: Beware that antivirus might flag the service or install scripts.**

Download the files from the latest release (recommended), or the build artifacts of the latest commit (not recommended).
Afterwards, copy both the autopower.exe, autopower_notification_provider.exe, and all .ps1 files from the folder to your desired location (recommended to be a non-admin folder).
Then simply run the install_service.ps1 script, and everything should work!
After this, the service will start automatically with your PC.

## Building

Important to build with the command `cargo build --release --workspace` so all binaries gets built.

## Uninstallation

Open the directory where you placed the service executable, then just run the uninstall_service.ps1 script. Note that a restart might be required to completely remove it.
Afterwards you can delete the directory.
