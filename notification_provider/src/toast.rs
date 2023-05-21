use autopower_shared::winstr::to_h_string;
use windows::{
    core::HSTRING,
    h,
    UI::Notifications::{ToastNotification, ToastNotificationManager, ToastTemplateType},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Toast<'a> {
    title: &'a str,
    description: &'a str,
}

impl<'a> Toast<'a> {
    pub fn new(title: &'a str, description: &'a str) -> Self {
        Self { title, description }
    }

    fn create_notifcation(&self) -> Result<ToastNotification> {
        let toast_xml =
            ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText01)
                .map_err(|e| format!("Could not get template content!\n{}", e))?;

        let string_elems = toast_xml
            .GetElementsByTagName(h!("text"))
            .map_err(|e| format!("Could not get elements by tag name!\n{}", e))?;

        for i in 0..string_elems.Length()? {
            let elem = string_elems
                .Item(i)
                .map_err(|e| format!("Could not get item!\n{}", e))?;
            let node = &toast_xml
                .CreateTextNode(&HSTRING::from(self.description))
                .map_err(|e| format!("Could not create text node!\n{}", e))?;
            elem.AppendChild(node)
                .map_err(|e| format!("Could not append child!\n{}", e))?;
        }

        let toast = ToastNotification::CreateToastNotification(&toast_xml)
            .map_err(|e| format!("Could not create toast notification!\n{}", e))?;
        Ok(toast)
    }

    pub fn send(&self) -> Result<()> {
        let toast = self.create_notifcation()?;
        toast
            .SetExpiresOnReboot(true)
            .map_err(|e| format!("Could not set expire on reboot!\n{}", e))?;

        let title = to_h_string(self.title)?;
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&title)
            .map_err(|e| format!("Could not create toast notifier with id!\n{}", e))?;
        notifier
            .Show(&toast)
            .map_err(|e| format!("Could not show toast!\n{}", e))?;
        Ok(())
    }
}
