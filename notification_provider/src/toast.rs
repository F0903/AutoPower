use autopower_shared::util::to_h_string;
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
            ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText01)?;

        let string_elems = toast_xml.GetElementsByTagName(h!("text"))?;

        for i in 0..string_elems.Length()? {
            let elem = string_elems.Item(i)?;
            elem.AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(self.description))?)?;
        }

        let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
        Ok(toast)
    }

    pub fn send(&self) -> Result<()> {
        let toast = self.create_notifcation()?;
        let title = to_h_string(self.title)?;
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&title)?;
        notifier.Show(&toast)?;
        Ok(())
    }
}
