use std::fs::{self, File, OpenOptions, Permissions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn open_for_wiping(path: &Path) -> io::Result<File> {
	OpenOptions::new().read(true).write(true).open(path)
}

pub fn obfuscate_metadata(file: &File) -> io::Result<()> {
	file.set_permissions(Permissions::from_mode(0o0))?;

	let times = [
		libc::timespec {
			tv_sec: 0,
			tv_nsec: 0,
		},
		libc::timespec {
			tv_sec: 0,
			tv_nsec: 0,
		},
	];
	let result = unsafe { libc::futimens(file.as_raw_fd(), times.as_ptr()) };
	if result == -1 {
		return Err(io::Error::last_os_error());
	}

	Ok(())
}

pub fn random_name(bytes: &[u8]) -> String {
	let mut name = String::with_capacity(2 + bytes.len() * 2);
	name.push_str(".wipe-");
	for byte in bytes {
		name.push_str(&format!("{byte:02x}"));
	}
	name
}

pub fn rename_for_removal(path: &Path, name: &str) -> io::Result<PathBuf> {
	let parent = path.parent().unwrap_or_else(|| Path::new("."));
	let renamed = parent.join(name);
	fs::rename(path, &renamed)?;
	Ok(renamed)
}

pub fn remove_file(path: &Path) -> io::Result<()> {
	fs::remove_file(path)
}

pub fn sync_directory(path: &Path) -> io::Result<()> {
	let parent = path.parent().unwrap_or_else(|| Path::new("."));
	File::open(parent)?.sync_all()
}
