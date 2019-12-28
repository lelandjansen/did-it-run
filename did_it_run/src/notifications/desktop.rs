use crate::notifications::{Dispatcher, NotificationInfo, NotifierError};
use crate::DID_IT_RUN_NAME;
use std::convert::From;
use std::error;
use std::fmt;
#[cfg(all(target_os = "windows", not(test)))]
use winrt::windows::data::xml::dom::IXmlNode;
#[cfg(all(target_os = "windows", not(test)))]
use winrt::windows::ui::notifications::{
    ToastNotification, ToastNotificationManager, ToastTemplateType,
};

#[cfg(all(target_os = "windows", not(test)))]
#[rustfmt::skip]
const TOAST_ID: &str = r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe";

#[derive(Debug)]
pub enum DesktopError {
    #[cfg(target_os = "linux")]
    AlreadyInitialized,
    #[cfg(target_os = "linux")]
    Glib(glib::error::Error),
    #[cfg(target_os = "linux")]
    Other(String),
    #[cfg(target_os = "macos")]
    MacOs(mac_notification_sys::error::Error),
    #[cfg(all(target_os = "windows", not(test)))]
    NoneError,
    #[cfg(target_os = "windows")]
    WinRt(winrt::Error),
}

pub struct DesktopNotifier {
    #[cfg(all(target_os = "windows", not(test)))]
    #[allow(dead_code)]
    // Take ownership of the runtime context which is automatically
    // uninitialized when dropped, but never used.
    context: winrt::RuntimeContext,
}

impl DesktopNotifier {
    #[cfg(all(target_os = "linux", not(test)))]
    pub fn new() -> Result<Self, DesktopError> {
        if libnotify::is_initted() {
            return Err(DesktopError::AlreadyInitialized);
        }
        libnotify::init(DID_IT_RUN_NAME)?;
        Ok(DesktopNotifier {})
    }

    #[cfg(all(target_os = "macos", not(test)))]
    pub fn new() -> Result<Self, DesktopError> {
        Ok(DesktopNotifier {})
    }

    #[cfg(all(target_os = "windows", not(test)))]
    pub fn new() -> Result<Self, DesktopError> {
        Ok(DesktopNotifier {
            context: winrt::RuntimeContext::init(),
        })
    }

    #[cfg(test)]
    pub fn new() -> Result<Self, DesktopError> {
        let _ = DID_IT_RUN_NAME;
        Ok(DesktopNotifier {})
    }
}

impl Dispatcher for DesktopNotifier {
    #[cfg(all(target_os = "linux", not(test)))]
    fn dispatch_notification(
        &mut self,
        info: NotificationInfo,
    ) -> Result<(), NotifierError> {
        let summary = DID_IT_RUN_NAME;
        let body = Some(info.brief.as_str());
        let icon = None;
        libnotify::Notification::new(summary, body, icon).show()?;
        Ok(())
    }

    #[cfg(all(target_os = "macos", not(test)))]
    fn dispatch_notification(
        &mut self,
        info: NotificationInfo,
    ) -> Result<(), NotifierError> {
        let title = DID_IT_RUN_NAME;
        let subtitle = &None;
        let message = &info.brief;
        let sound = &None;
        mac_notification_sys::send_notification(
            title, subtitle, message, sound,
        )?;
        Ok(())
    }

    #[cfg(all(target_os = "windows", not(test)))]
    fn dispatch_notification(
        &mut self,
        info: NotificationInfo,
    ) -> Result<(), NotifierError> {
        // TODO(#30): Replace `ok_or` with the `?` shorthand and
        // `From<std::option::NoneError>` when the `try_trait` API stabilizes.
        let toast_xml = ToastNotificationManager::get_template_content(
            ToastTemplateType::ToastText02,
        )?
        .ok_or(DesktopError::NoneError)?;
        let toast_text_elements = toast_xml
            .get_elements_by_tag_name(&winrt::FastHString::new("text"))?
            .ok_or(DesktopError::NoneError)?;
        toast_text_elements
            .item(0)?
            .ok_or(DesktopError::NoneError)?
            .append_child(
                &*toast_xml
                    .create_text_node(&winrt::FastHString::from(
                        DID_IT_RUN_NAME,
                    ))?
                    .ok_or(DesktopError::NoneError)?
                    .query_interface::<IXmlNode>()
                    .ok_or(DesktopError::NoneError)?,
            )?
            .ok_or(DesktopError::NoneError)?;
        toast_text_elements
            .item(1)?
            .ok_or(DesktopError::NoneError)?
            .append_child(
                &*toast_xml
                    .create_text_node(&winrt::FastHString::from(
                        info.brief.as_str(),
                    ))?
                    .ok_or(DesktopError::NoneError)?
                    .query_interface::<IXmlNode>()
                    .ok_or(DesktopError::NoneError)?,
            )?
            .ok_or(DesktopError::NoneError)?;

        let toast = ToastNotification::create_toast_notification(&*toast_xml)?;
        ToastNotificationManager::create_toast_notifier_with_id(
            &winrt::FastHString::new(TOAST_ID),
        )?
        .ok_or(DesktopError::NoneError)?
        .show(&*toast)?;
        Ok(())
    }

    #[cfg(test)]
    fn dispatch_notification(
        &mut self,
        _: NotificationInfo,
    ) -> Result<(), NotifierError> {
        Ok(())
    }
}

#[cfg(all(target_os = "linux", not(test)))]
impl Drop for DesktopNotifier {
    fn drop(&mut self) {
        if libnotify::is_initted() {
            libnotify::uninit();
        }
    }
}

impl error::Error for DesktopError {}

impl fmt::Display for DesktopError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self)
    }
}

#[cfg(target_os = "linux")]
impl From<glib::error::Error> for NotifierError {
    fn from(err: glib::error::Error) -> Self {
        NotifierError::Desktop(DesktopError::Glib(err))
    }
}

#[cfg(target_os = "linux")]
impl From<String> for DesktopError {
    fn from(err: String) -> Self {
        DesktopError::Other(err)
    }
}

#[cfg(target_os = "macos")]
impl From<mac_notification_sys::error::Error> for NotifierError {
    fn from(err: mac_notification_sys::error::Error) -> Self {
        NotifierError::Desktop(DesktopError::MacOs(err))
    }
}

#[cfg(target_os = "windows")]
impl From<winrt::Error> for NotifierError {
    fn from(err: winrt::Error) -> Self {
        NotifierError::Desktop(DesktopError::WinRt(err))
    }
}
