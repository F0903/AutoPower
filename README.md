# AutoPower

A small and efficient Windows service that automatically sets your laptops Power Mode to High Performance when you plug it into the wall.
Then when you plug it out, it will automatically set your laptop back to Balanced.

Uses virtually no CPU, as code only runs when you plug your laptop in or out of power. It also allocates minimal memory, so you don't need to worry about any performance impact when the service is running.

## Installation

Simply clone the project and build. Afterwards, copy the release output folder to your desired location, then simply run the install_service script, and everything should work!
Note that a manual start is not required again after this, as it will now automatically start with your PC.

## Uninstallation

Open the folder where you placed the service, here you should have saved the provided scripts from the Install step.
Then just run the uninstall_service script. Note that a restart might be required to completely remove it. 
Afterwards you can delete the folder.

__Note:__
__This program has only been tested with Windows 11 and 10, but should work down to Windows 7.__
