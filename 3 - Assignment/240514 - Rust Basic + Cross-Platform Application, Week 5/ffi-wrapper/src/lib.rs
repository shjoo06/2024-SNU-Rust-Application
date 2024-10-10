// TODO: remove this when you're done with your implementation.
#![allow(unused_imports, unused_variables, dead_code)]

// C uses \0 as null terminator to indicate the end of string,
// while Rust explicitly stores length of the string without using null terminator.
// Rust -> C: String -> CString(\0-terminated, C-friendly string) -> *mut u8 (raw char array) -> pass to C function ...
// C -> Rust: C function -> CStr wrapping raw *const u8 -> &str (check if utf-8 encoding)

mod ffi {
    use std::os::raw::{c_char, c_int};
    #[cfg(not(target_os = "macos"))]
    use std::os::raw::{c_long, c_uchar, c_ulong, c_ushort};

    // Opaque type. See https://doc.rust-lang.org/nomicon/ffi.html.
    // 'opaque' struct: its contents are not part of the public interface; gives some amount of type safety
    #[repr(C)] // order, size, and alignment of fields as in C/C++. Any type you expect to pass through an FFI boundary should have repr(C)
    pub struct DIR {
        _data: [u8; 0],
        _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
    }

    // Layout according to the Linux man page for readdir(3), where ino_t and
    // off_t are resolved according to the definitions in
    // /usr/include/x86_64-linux-gnu/{sys/types.h, bits/typesizes.h}.
    #[cfg(not(target_os = "macos"))]
    #[repr(C)]
    pub struct dirent {
        pub d_ino: c_ulong,
        pub d_off: c_long,
        pub d_reclen: c_ushort,
        pub d_type: c_uchar,
        pub d_name: [c_char; 256],
    }

    // Layout according to the macOS man page for dir(5).
    #[cfg(all(target_os = "macos"))]
    #[repr(C)]
    pub struct dirent {
        pub d_fileno: u64,
        pub d_seekoff: u64,
        pub d_reclen: u16,
        pub d_namlen: u16,
        pub d_type: u8,
        pub d_name: [c_char; 1024],
    }

    extern "C" { // extern "C" makes this function adhere to the C calling convention
        // declaring function interfaces that Rust code can call foreign code by
        pub fn opendir(s: *const c_char) -> *mut DIR;

        #[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
        pub fn readdir(s: *mut DIR) -> *const dirent;

        // See https://github.com/rust-lang/libc/issues/414 and the section on
        // _DARWIN_FEATURE_64_BIT_INODE in the macOS man page for stat(2).
        //
        // "Platforms that existed before these updates were available" refers
        // to macOS (as opposed to iOS / wearOS / etc.) on Intel and PowerPC.
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        #[link_name = "readdir$INODE64"]
        pub fn readdir(s: *mut DIR) -> *const dirent;

        pub fn closedir(s: *mut DIR) -> c_int;
    }
}

use std::ffi::{CStr, CString, OsStr, OsString};
use std::os::unix::ffi::OsStrExt;

#[derive(Debug)]
struct DirectoryIterator {
    path: CString,
    dir: *mut ffi::DIR,
}

impl DirectoryIterator {
    fn new(path: &str) -> Result<DirectoryIterator, String> {
        // Call opendir and return a Ok value if that worked,
        // otherwise return Err with a message.
        let path_cstring = CString::new(path).expect("failed to create CString");
        // let ptr = path_cstring.clone().into_raw(); // in this case, ptr: *mut c_char
        // https://doc.rust-lang.org/std/ffi/struct.CString.html#extracting-a-raw-pointer-to-the-whole-c-string
        let ptr = path_cstring.as_ptr(); // read-only pointer (*const c_char)
        let dir = unsafe { ffi::opendir(ptr) };
        if dir.is_null() { // opendir error check
            return Err(format!("Failed to open directory: {}", path));
        }
        let entry: Self = DirectoryIterator
        { path: path_cstring,
            dir };
        /* SAFETY: This function is called with a pointer that was obtained
        by calling CString::into_raw. */
        // unsafe { let _ = CString::from_raw(ptr); }; // retake pointer to free memory (in case of using .into_raw())
        Ok(entry)
    }
}

impl Iterator for DirectoryIterator {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> {
        // Keep calling readdir until we get a NULL pointer back.
        unsafe{
            let dirent_ptr = ffi::readdir(self.dir);
            if dirent_ptr.is_null() {
                None
            } else {
                // C function return (raw *const u8) -> CStr -> OsString
                let ptr = (*dirent_ptr).d_name.as_ptr();
                let dname = CStr::from_ptr(ptr);
                let bytes = dname.to_bytes();
                let a = OsString::from_encoded_bytes_unchecked(Vec::from(bytes));
                Some(a)
            }
        }
    }
}

impl Drop for DirectoryIterator {
    fn drop(&mut self) {
        // Call closedir as needed.
        unsafe { let _ = ffi::closedir(self.dir); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_nonexisting_directory() {
        let iter = DirectoryIterator::new("no-such-directory");
        assert!(iter.is_err());
    }

    #[test]
    fn test_empty_directory() -> Result<(), Box<dyn Error>> {
        let tmp = tempfile::TempDir::new()?;
        let iter = DirectoryIterator::new(
            tmp.path().to_str().ok_or("Non UTF-8 character in path")?,
        )?;
        let mut entries = iter.collect::<Vec<_>>();

        entries.sort();

        assert_eq!(entries, &[".", ".."]);

        Ok(())
    }

    #[test]
    fn test_nonempty_directory() -> Result<(), Box<dyn Error>> {
        let tmp = tempfile::TempDir::new()?;
        std::fs::write(tmp.path().join("foo.txt"), "The Foo Diaries\n")?;
        std::fs::write(tmp.path().join("bar.png"), "<PNG>\n")?;
        std::fs::write(tmp.path().join("crab.rs"), "//! Crab\n")?;

        let iter = DirectoryIterator::new(
            tmp.path().to_str().ok_or("Non UTF-8 character in path")?,
        )?;
        let mut entries = iter.collect::<Vec<_>>();

        entries.sort();
        
        assert_eq!(entries, &[".", "..", "bar.png", "crab.rs", "foo.txt"]);

        Ok(())
    }
}
