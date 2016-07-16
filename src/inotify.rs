extern crate libc;
use self::libc::{
	c_int,
	c_char,
	c_void,
	size_t,
	__errno_location,
//	close,
	read,
	strnlen,
	EMFILE,
	ENFILE,
	ENOMEM,
	EACCES,
	EBADF,
	EFAULT,
	EINVAL,
	ENOENT,
	ENOSPC,
};
use std::path::Path;
use std::ffi::CString;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::mem::size_of;
use std::slice;

const IN_CLOSE_WRITE: u32 = 0x00000008;
const IN_MOVED_TO:    u32 = 0x00000080;

fn errno() -> c_int {
	unsafe {
		(*__errno_location()) as i32
	}
}

extern {
	pub fn inotify_init() -> c_int;
	pub fn inotify_add_watch(fd: c_int, pathname: *const c_char, mask: u32) -> c_int;
//	pub fn inotify_rm_watch(fd: c_int, wd: c_int) -> c_int;
}

#[repr(C)]
pub struct InotifyEvent {
    pub wd: c_int,
    pub mask: u32,
    pub cookie: u32,
    pub len: u32,
	pub name: [u8; 0]
}

pub struct InotifyInstance {
	fd: c_int,
	#[allow(dead_code)]	//todo
	watches: Vec<InotifyWatch>,
}

pub struct InotifyWatch {
	//instance: &InotifyInstance,
	#[allow(dead_code)]	//todo
	wd: c_int,
}

impl InotifyInstance {
	pub fn new() -> Result<InotifyInstance, &'static str> {
		let fd = unsafe{ inotify_init() };
		if fd == -1 {
			match errno() {
				EMFILE => Err("The user limit on the total number of inotify instances has been reached"),
				ENFILE => Err("The system limit on the total number of file descriptors has been reached"),
				ENOMEM => Err("Insufficient kernel memory is available"),
				_ => Err("Unknown error"),
			}
		} else {
			Ok(InotifyInstance{fd: fd, watches: Vec::with_capacity(32)})
		}
	}
	
	pub fn add_watch(&mut self, path: &Path) -> Result<InotifyWatch, &'static str> {
		let cpath = CString::new(path.as_os_str().as_bytes()).unwrap().as_ptr();
		let wd = unsafe{ inotify_add_watch(self.fd, cpath, IN_CLOSE_WRITE | IN_MOVED_TO) };
		if wd == -1 {
			match errno() {
				EACCES => Err("Read access to the given file is not permitted"),
				EBADF  => Err("The given file descriptor is not valid"),
				EFAULT => Err("pathname points outside of the process's accessible address space"),
				EINVAL => Err("The given event mask contains no valid events; or fd is not an inotify file descriptor"),
				ENOENT => Err("A directory component in pathname does not exist or is a dangling symbolic link"),
				ENOMEM => Err("Insufficient kernel memory was available"),
				ENOSPC => Err("The user limit on the total number of inotify watches was reached or the kernel failed to allocate a needed resource"),
				_ => Err("Unknown error"),
			}
		} else {
			//Ok(InotifyWatch{instance: self, wd: wd})
			Ok(InotifyWatch{wd: wd})
		}
	}
	
	pub fn process_events<F>(&self, mut process: F) where F: FnMut(&Path) {	//todo: investigate what's with FnMut
		let mut buffer = [0 as u8; 512];	//todo: use std::mem::size_of::<inotify_event> when it becomes available at compile time
		loop {
			let rr = unsafe { read(self.fd, buffer.as_mut_ptr() as *mut c_void, buffer.len() as size_t) };
			if rr < 0 {
				panic!("The specified buffer was too small to read at least one event. Increase the buffer size!");
			}
			let mut offset: usize = 0;
			while offset < rr as usize {
				let event = (&buffer[offset..]).as_ptr() as *const InotifyEvent;
				let padded_name_len = unsafe { (*event).len };
				let name_len = unsafe { strnlen(&(*event).name as *const u8 as *const c_char, padded_name_len as size_t) as usize };
				let name_slice = unsafe { slice::from_raw_parts(&(*event).name as *const u8, name_len) };
				let filename = Path::new(OsStr::from_bytes(name_slice));
				process(&filename);
				
				offset += size_of::<InotifyEvent>() + padded_name_len as usize;
			}
		}
	}
}

// impl InotifyWatch {
// 	fn cancel(&self) -> Result<?, &'static str> {
// 		unsafe{ inotify_rm_watch(self.instance.fd, self.wd) };
// 	}
// }
