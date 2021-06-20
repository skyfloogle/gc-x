use std::ops::DerefMut;
use std::pin::Pin;
use std::ptr;
use vigem::notification::X360Notification;
use vigem::raw::{LPVOID, PVIGEM_CLIENT, PVIGEM_TARGET, UCHAR};
use vigem::{Target, Vigem, VigemError};

pub struct NotificationHandle<'a> {
    target: PVIGEM_TARGET,
    func: Box<dyn FnMut(X360Notification<()>) + 'a>,
}

pub fn register_notification<'a, F: 'a + FnMut(X360Notification<()>)>(
    vigem: &mut Vigem,
    target: &Target,
    func: F,
) -> Result<Pin<Box<NotificationHandle<'a>>>, VigemError> {
    unsafe extern "C" fn handle_notification(
        arg1: PVIGEM_CLIENT,
        arg2: PVIGEM_TARGET,
        arg3: UCHAR,
        arg4: UCHAR,
        arg5: UCHAR,
        arg6: LPVOID,
    ) {
        let handle = &mut *arg6.cast::<NotificationHandle>();
        let notification = X360Notification::new(arg1, arg2, arg3, arg4, arg5, ptr::null_mut());
        (handle.func)(notification);
    }
    unsafe {
        let func: Box<dyn FnMut(X360Notification<()>)> = Box::new(func);
        let func: Box<dyn FnMut(X360Notification<()>) + 'static> = std::mem::transmute(func);
        let mut handle = Pin::new(Box::new(NotificationHandle {
            target: *target.raw,
            func,
        }));
        let err = vigem::raw::vigem_target_x360_register_notification(
            **vigem.vigem,
            *target.raw,
            Some(handle_notification),
            handle.deref_mut() as *mut NotificationHandle as _,
        );
        VigemError::new(err).to_result().map(|()| handle)
    }
}

impl Drop for NotificationHandle<'_> {
    fn drop(&mut self) {
        unsafe {
            vigem::raw::vigem_target_x360_unregister_notification(self.target);
        }
    }
}
