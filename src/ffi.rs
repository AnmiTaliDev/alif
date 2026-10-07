// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::missing_safety_doc)]

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::Path;

use crate::error::AlifError;
use crate::{platform_warning, verify_file, verify_source};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(message: Option<String>) {
    let value = message.map(|m| CString::new(m.replace('\0', " ")).expect("NUL bytes removed"));
    LAST_ERROR.with(|slot| *slot.borrow_mut() = value);
}

fn finish(result: Result<(), AlifError>) -> c_int {
    match result {
        Ok(()) => {
            set_last_error(None);
            0
        }
        Err(error) => {
            let code = match error {
                AlifError::Check(_) => 1,
                AlifError::Parse(_) | AlifError::Load(_) => 2,
            };
            set_last_error(Some(error.to_string()));
            code
        }
    }
}

unsafe fn read_argument<'a>(pointer: *const c_char) -> Result<&'a str, c_int> {
    if let Some(warning) = platform_warning() {
        eprintln!("{}", warning);
    }
    if pointer.is_null() {
        set_last_error(Some("null pointer argument".to_string()));
        return Err(2);
    }
    match unsafe { CStr::from_ptr(pointer) }.to_str() {
        Ok(text) => Ok(text),
        Err(_) => {
            set_last_error(Some("argument is not valid UTF-8".to_string()));
            Err(2)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn alif_verify(source: *const c_char) -> c_int {
    match unsafe { read_argument(source) } {
        Ok(text) => finish(verify_source(text)),
        Err(code) => code,
    }
}

#[no_mangle]
pub unsafe extern "C" fn alif_verify_file(path: *const c_char) -> c_int {
    match unsafe { read_argument(path) } {
        Ok(text) => finish(verify_file(Path::new(text))),
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "C" fn alif_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| match &*slot.borrow() {
        Some(message) => message.as_ptr(),
        None => std::ptr::null(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn last_error() -> Option<String> {
        let pointer = alif_last_error();
        if pointer.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(pointer) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    fn verify(text: &str) -> c_int {
        let source = CString::new(text).unwrap();
        unsafe { alif_verify(source.as_ptr()) }
    }

    #[test]
    fn valid_source_returns_zero_and_clears_error() {
        assert_eq!(verify("!!!"), 2);
        assert!(last_error().is_some());
        assert_eq!(
            verify("theorem t: A |- A\nproof\n  assume h: A\n  exact h\nqed"),
            0
        );
        assert_eq!(last_error(), None);
    }

    #[test]
    fn proof_error_returns_one() {
        let code = verify("theorem t: A |- B\nproof\n  assume h: A\n  exact h\nqed");
        assert_eq!(code, 1);
        assert!(last_error().unwrap().contains("proof error"));
    }

    #[test]
    fn parse_error_returns_two() {
        assert_eq!(verify("!!!"), 2);
        assert!(last_error().unwrap().contains("parse error"));
    }

    #[test]
    fn null_pointer_returns_two() {
        assert_eq!(unsafe { alif_verify(std::ptr::null()) }, 2);
        assert!(last_error().unwrap().contains("null"));
    }

    #[test]
    fn invalid_utf8_returns_two() {
        let bytes = [0xff_u8, 0xfe, 0x00];
        let code = unsafe { alif_verify(bytes.as_ptr() as *const c_char) };
        assert_eq!(code, 2);
        assert!(last_error().unwrap().contains("UTF-8"));
    }

    #[test]
    fn missing_file_returns_two() {
        let path = CString::new("/nonexistent/alif-missing.alif").unwrap();
        assert_eq!(unsafe { alif_verify_file(path.as_ptr()) }, 2);
        assert!(last_error().unwrap().contains("cannot read"));
    }

    #[test]
    fn file_verification_returns_zero() {
        let path = std::env::temp_dir().join(format!("alif-ffi-{}.alif", std::process::id()));
        std::fs::write(&path, "theorem t: A |- A\nproof\n  assume h: A\n  exact h\nqed").unwrap();
        let c_path = CString::new(path.to_str().unwrap()).unwrap();
        assert_eq!(unsafe { alif_verify_file(c_path.as_ptr()) }, 0);
        std::fs::remove_file(&path).unwrap();
    }
}
