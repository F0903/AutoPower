# AutoPower

A small and efficient Windows service that automatically switches your laptop between the Balanced and High Performance power modes when you plug it in or out of power, and sends you a notification.

Uses virtually no CPU, as code only runs when you plug your laptop in or out of power. It also allocates minimal memory, so you don't need to worry about any performance impact when the service is running.

## Installation

**Note: Only Windows 11/10 is supported due to API requirements.**  
**Note: Beware that antivirus might flag the service or install scripts.**

Simply clone the project and build with the command `cargo build --release --workspace`.
Afterwards, copy both the autopower.exe, autopower_notification_provider.exe, and .ps1 files from the ./target/release directory to your desired location, then simply run the install_service.ps1 script, and everything should work!
After this, the service will start automatically with your PC.

## Uninstallation

Open the directory where you placed the service executable, here you should have saved the provided scripts from the Installation step.
Then just run the uninstall_service.ps1 script. Note that a restart might be required to completely remove it.
Afterwards you can delete the directory.

If you did not save the installation scripts, open a PowerShell session with Admin rights. Then just type `sc.exe delete AutoPower` and follow as above.
