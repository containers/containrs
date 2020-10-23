//! error handling FFI interface

use anyhow::Error;
use libc::{c_char, c_int};
use log::{error, warn};
use std::{cell::RefCell, ptr, slice};

thread_local! {
    static LAST_ERROR: RefCell<Option<Error>> = RefCell::new(None);
}

#[allow(dead_code)]
/// Update the last error by the provided one.
pub fn update_last_error(err: Error) {
    error!("Setting last error: {:#}", err);

    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(err);
    });
}

/// Calculate the number of bytes in the last error's error message including a trailing `null`
/// character. If there are no recent error, then this returns `0`.
#[no_mangle]
pub extern "C" fn last_error_length() -> c_int {
    LAST_ERROR.with(|prev| match *prev.borrow() {
        Some(ref err) => format!("{:#}", err).len() as c_int + 1,
        None => 0,
    })
}

/// Write the most recent error message into a caller-provided buffer as a UTF-8
/// string, returning the number of bytes written.
///
/// # Note
///
/// This writes a **UTF-8** string into the buffer. Windows users may need to
/// convert it to a UTF-16 "unicode" afterwards.
///
/// If there are no recent errors then this returns `0` (because we wrote 0
/// bytes). `-1` is returned if there are argument based errors, for example
/// when passed a `null` pointer or a buffer of insufficient size.
#[no_mangle]
pub extern "C" fn last_error_message(buffer: *mut c_char, length: c_int) -> c_int {
    if buffer.is_null() {
        warn!("Null pointer passed into last_error_message() as the buffer");
        return -1;
    }

    // Retrieve the most recent error, clearing it in the process.
    let last_error = match LAST_ERROR.with(|prev| prev.borrow_mut().take()) {
        Some(err) => err,
        None => return 0,
    };

    let error_message = format!("{:#}", last_error);
    let buffer = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, length as usize) };

    if error_message.len() >= buffer.len() {
        warn!("Buffer provided for writing the last error message is too small");
        warn!(
            "Expected at least {} bytes but got {}",
            error_message.len() + 1,
            buffer.len()
        );
        return -1;
    }

    unsafe {
        ptr::copy_nonoverlapping(
            error_message.as_ptr(),
            buffer.as_mut_ptr(),
            error_message.len(),
        )
    };

    // Add a trailing null so people using the string as a `char *` don't
    // accidentally read into garbage.
    buffer[error_message.len()] = 0;

    error_message.len() as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};

    #[test]
    fn update_last_error_success() -> Result<()> {
        let mut buf = Vec::with_capacity(100);

        // No error
        assert!(LAST_ERROR.with(|prev| prev.borrow().is_none()));
        assert_eq!(last_error_length(), 0);
        assert_eq!(last_error_message(buf.as_mut_ptr() as *mut c_char, 0), 0);

        // Some error
        let err = anyhow!("some error");
        update_last_error(err.context("some other error"));
        assert!(LAST_ERROR.with(|prev| prev.borrow().is_some()));
        assert_eq!(last_error_length(), 29);

        // But no buffer
        assert_eq!(last_error_message(ptr::null_mut(), 0), -1);

        // Or buffer is too small
        assert_eq!(last_error_message(buf.as_mut_ptr() as *mut c_char, 1), -1);

        // Error already taken
        assert!(LAST_ERROR.with(|prev| prev.borrow().is_none()));
        assert_eq!(last_error_length(), 0);

        // Insert new error
        update_last_error(anyhow!("new error"));
        assert!(LAST_ERROR.with(|prev| prev.borrow().is_some()));
        assert_eq!(last_error_length(), 10);

        // Buffer is large enough
        assert_eq!(last_error_message(buf.as_mut_ptr() as *mut c_char, 100), 9);

        Ok(())
    }
}
