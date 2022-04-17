A small and efficient Windows service that automatically sets your laptops Power Mode to High Performance when you plug it into the wall.
Then when you plug it out, it will automatically set your laptop back to Balanced.

Uses virtually no CPU, as code only runs when you plug your laptop in or out of power. It also allocates minimal memory, so you don't need to worry about any performance impact when the service is running.

To install, you will have to compile the program yourself (for now), then move the compiled .exe into your desired directory and remember the path.
Then open your terminal of choice (with admin perms), and type in the following: ```sc create "AutoPower" binPath="*your path to the .exe*"```
After that it should be successfully created. 
Now you can type "Services" in the Windows search bar, and you should be able to open the Service Manager.
From here you can locate "AutoPower", and start it. Alternatively, you can also go into its properties, and set it to start with Windows (recommended ;).

If you ever want to uninstall, simply open your terminal again (with admin perms!), and type ```sc delete AutoPower```. 
That should mark it for deletion, and it will be gone after a restart. (then you can delete the executeable)
