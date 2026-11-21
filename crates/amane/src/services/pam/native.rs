use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use std::sync::Arc;

use super::config::PreparedConfig;
use super::{PamError, PamMessageKind, PamResult, Session};

const NO_FLAGS: c_int = 0;

const PAM_SUCCESS: c_int = 0;
const PAM_BUF_ERR: c_int = 5;
const PAM_PERM_DENIED: c_int = 6;
const PAM_AUTH_ERR: c_int = 7;
const PAM_USER_UNKNOWN: c_int = 10;
const PAM_MAXTRIES: c_int = 11;
const PAM_NEW_AUTHTOK_REQD: c_int = 12;
const PAM_ACCT_EXPIRED: c_int = 13;
const PAM_CONV_ERR: c_int = 19;

const PAM_USER: c_int = 2;

const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_PROMPT_ECHO_ON: c_int = 2;
const PAM_ERROR_MSG: c_int = 3;
const PAM_TEXT_INFO: c_int = 4;
const PAM_MAX_NUM_MSG: c_int = 32;

#[repr(C)]
struct Message {
    style: c_int,
    text: *const c_char,
}

#[repr(C)]
struct Response {
    text: *mut c_char,
    code: c_int,
}

type Converse =
    unsafe extern "C" fn(c_int, *mut *const Message, *mut *mut Response, *mut c_void) -> c_int;

#[repr(C)]
struct Conversation {
    converse: Converse,
    data: *mut c_void,
}

#[link(name = "pam")]
unsafe extern "C" {
    fn pam_start(
        service: *const c_char,
        user: *const c_char,
        conversation: *const Conversation,
        handle: *mut *mut c_void,
    ) -> c_int;
    fn pam_start_confdir(
        service: *const c_char,
        user: *const c_char,
        conversation: *const Conversation,
        directory: *const c_char,
        handle: *mut *mut c_void,
    ) -> c_int;
    fn pam_authenticate(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_acct_mgmt(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_get_item(handle: *const c_void, item: c_int, value: *mut *const c_void) -> c_int;
    fn pam_strerror(handle: *mut c_void, code: c_int) -> *const c_char;
    fn pam_end(handle: *mut c_void, status: c_int) -> c_int;
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn strdup(text: *const c_char) -> *mut c_char;
    fn free(pointer: *mut c_void);
}

struct Handle {
    pointer: *mut c_void,
    status: c_int,
}

impl Drop for Handle {
    fn drop(&mut self) {
        // only a successfully started handle is owned, and its conversation is still alive
        unsafe {
            pam_end(self.pointer, self.status);
        }
    }
}

struct Context<'a> {
    session: &'a Arc<Session>,
    password: Option<&'a str>,
}

pub(super) fn authenticate(
    config: &PreparedConfig,
    session: &Arc<Session>,
    password: Option<&str>,
) -> PamResult {
    let mut context = Context { session, password };
    let conversation = Conversation {
        converse,
        data: ptr::from_mut(&mut context).cast(),
    };
    let mut pointer = ptr::null_mut();
    // all strings and the callback context outlive the handle; PAM stays on this worker thread
    let status = unsafe {
        match &config.directory {
            Some(directory) => pam_start_confdir(
                config.service.as_ptr(),
                config.user.as_ptr(),
                &conversation,
                directory.as_ptr(),
                &mut pointer,
            ),
            None => pam_start(
                config.service.as_ptr(),
                config.user.as_ptr(),
                &conversation,
                &mut pointer,
            ),
        }
    };
    if status != PAM_SUCCESS {
        return classify(ptr::null_mut(), status);
    }
    let mut handle = Handle { pointer, status };
    if session.aborted() {
        return PamResult::Aborted;
    }
    handle.status = unsafe { pam_authenticate(handle.pointer, NO_FLAGS) };
    if session.aborted() {
        return PamResult::Aborted;
    }
    if handle.status == PAM_SUCCESS {
        // accepting a password must not bypass account expiry or access restrictions
        handle.status = unsafe { pam_acct_mgmt(handle.pointer, NO_FLAGS) };
    }
    if session.aborted() {
        return PamResult::Aborted;
    }
    if handle.status != PAM_SUCCESS {
        return classify(handle.pointer, handle.status);
    }

    let mut user = ptr::null();
    handle.status = unsafe { pam_get_item(handle.pointer, PAM_USER, &mut user) };
    if handle.status != PAM_SUCCESS {
        return classify(handle.pointer, handle.status);
    }
    // a module can change PAM_USER; success for someone else must not unlock this user's session
    if user.is_null() || unsafe { CStr::from_ptr(user.cast()) } != config.user.as_c_str() {
        handle.status = PAM_AUTH_ERR;
        return PamResult::Error(PamError::UserChanged);
    }
    PamResult::Success
}

fn classify(handle: *mut c_void, code: c_int) -> PamResult {
    let text = unsafe { pam_strerror(handle, code) };
    let message = if text.is_null() {
        String::from("unknown error")
    } else {
        unsafe { CStr::from_ptr(text) }
            .to_string_lossy()
            .into_owned()
    };
    let error = PamError::Native { code, message };
    match code {
        // policy or credential rejection, distinct from a broken/unavailable PAM backend
        PAM_PERM_DENIED | PAM_AUTH_ERR | PAM_USER_UNKNOWN | PAM_MAXTRIES | PAM_NEW_AUTHTOK_REQD
        | PAM_ACCT_EXPIRED => PamResult::Rejected(error),
        _ => PamResult::Error(error),
    }
}

fn wipe_string(s: &mut str) {
    let bytes = unsafe { s.as_bytes_mut() };
    for b in bytes.iter_mut() {
        unsafe { std::ptr::write_volatile(b, 0) };
    }
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}

fn wipe_cstring(s: CString) {
    let mut bytes = s.into_bytes_with_nul();
    for b in bytes.iter_mut() {
        unsafe { std::ptr::write_volatile(b, 0) };
    }
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}

unsafe fn libc_strlen(mut s: *const c_char) -> usize {
    let mut len = 0;
    unsafe {
        while *s != 0 {
            len += 1;
            s = s.add(1);
        }
    }
    len
}

// responses use the C allocator because the PAM module frees them after a successful callback
struct Answers {
    pointer: *mut Response,
    count: usize,
}

impl Answers {
    fn release(mut self) -> *mut Response {
        std::mem::replace(&mut self.pointer, ptr::null_mut())
    }
}

impl Drop for Answers {
    fn drop(&mut self) {
        if self.pointer.is_null() {
            return;
        }
        unsafe {
            for index in 0..self.count {
                let text_ptr = (*self.pointer.add(index)).text;
                if !text_ptr.is_null() {
                    let len = libc_strlen(text_ptr);
                    for i in 0..len {
                        std::ptr::write_volatile(text_ptr.add(i) as *mut u8, 0);
                    }
                    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
                    free(text_ptr.cast());
                }
            }
            free(self.pointer.cast());
        }
    }
}

unsafe extern "C" fn converse(
    count: c_int,
    messages: *mut *const Message,
    responses: *mut *mut Response,
    data: *mut c_void,
) -> c_int {
    // a Rust panic must never cross PAM's C stack; partial response allocations have their own guard
    panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        answer(count, messages, responses, data)
    }))
    .unwrap_or(PAM_CONV_ERR)
}

unsafe fn answer(
    count: c_int,
    messages: *mut *const Message,
    responses: *mut *mut Response,
    data: *mut c_void,
) -> c_int {
    if !(1..=PAM_MAX_NUM_MSG).contains(&count)
        || messages.is_null()
        || responses.is_null()
        || data.is_null()
    {
        return PAM_CONV_ERR;
    }
    let count = count as usize;
    let pointer = unsafe { calloc(count, size_of::<Response>()) }.cast::<Response>();
    if pointer.is_null() {
        return PAM_BUF_ERR;
    }
    let answers = Answers { pointer, count };
    let context = unsafe { &*data.cast::<Context<'_>>() };
    for index in 0..count {
        // Linux-PAM supplies an array of message pointers, not a contiguous Message array
        let pointer = unsafe { *messages.add(index) };
        if pointer.is_null() {
            return PAM_CONV_ERR;
        }
        let message = unsafe { &*pointer };
        let kind = match message.style {
            PAM_PROMPT_ECHO_OFF => PamMessageKind::Prompt { visible: false },
            PAM_PROMPT_ECHO_ON => PamMessageKind::Prompt { visible: true },
            PAM_ERROR_MSG => PamMessageKind::Error,
            PAM_TEXT_INFO => PamMessageKind::Info,
            _ => return PAM_CONV_ERR,
        };
        if message.text.is_null() {
            return PAM_CONV_ERR;
        }
        let text = unsafe { CStr::from_ptr(message.text) }
            .to_string_lossy()
            .into_owned();
        let Ok(response) = super::message(context.session, text, kind, context.password) else {
            return PAM_CONV_ERR;
        };
        if let Some(mut response) = response {
            let Ok(c_response) = CString::new(response.as_str()) else {
                wipe_string(response.as_mut_str());
                return PAM_CONV_ERR;
            };
            wipe_string(response.as_mut_str());

            let text = unsafe { strdup(c_response.as_ptr()) };
            wipe_cstring(c_response);

            if text.is_null() {
                return PAM_BUF_ERR;
            }
            unsafe {
                (*answers.pointer.add(index)).text = text;
            }
        }
    }
    unsafe {
        *responses = answers.release();
    }
    PAM_SUCCESS
}
