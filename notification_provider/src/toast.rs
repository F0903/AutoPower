use windows::{
    core::HSTRING,
    h,
    Foundation::TypedEventHandler,
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
            ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02)?;

        let string_elems = toast_xml.GetElementsByTagName(h!("text"))?;

        string_elems
            .Item(0)?
            .AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(self.title))?)?;

        for i in 1..string_elems.Length()? {
            let elem = string_elems.Item(i)?;
            elem.AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(self.description))?)?;
        }

        let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
        Ok(toast)
    }

    pub fn send(&self) -> Result<()> {
        let toast = self.create_notifcation()?;
        toast.Activated(&TypedEventHandler::new(|_, _| Ok(())))?;
        ToastNotificationManager::CreateToastNotifier()?.Show(&toast)?;
        Ok(())
    }
}
