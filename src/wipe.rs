use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::pattern::{Pattern, PatternSpec, RandomPattern};
use crate::platform::linux;

const CHUNK_SIZE: usize = 1024 * 1024;
const RENAME_BYTES: usize = 16;

pub fn wipe_paths(paths: &[PathBuf], recursive: bool, passes: &[PatternSpec]) -> io::Result<()> {
	for path in paths {
		let metadata = fs::symlink_metadata(path)?;
		if metadata.is_dir() {
			if !recursive {
				return Err(io::Error::new(
					io::ErrorKind::InvalidInput,
					format!("directory requires --recursive: {}", path.display()),
				));
			}
			wipe_directory(path, metadata.dev(), passes)?;
		} else if metadata.is_file() {
			wipe_file(path, passes)?;
		} else {
			return Err(unsupported_target(path));
		}
	}

	Ok(())
}

fn wipe_directory(path: &Path, root_device: u64, passes: &[PatternSpec]) -> io::Result<()> {
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let entry_path = entry.path();
		let metadata = fs::symlink_metadata(&entry_path)?;

		if metadata.file_type().is_symlink() {
			continue;
		}
		if metadata.is_dir() {
			if metadata.dev() == root_device {
				wipe_directory(&entry_path, root_device, passes)?;
			}
			continue;
		}
		if metadata.is_file() {
			wipe_file(&entry_path, passes)?;
			continue;
		}
		return Err(unsupported_target(&entry_path));
	}

	Ok(())
}

fn wipe_file(path: &Path, passes: &[PatternSpec]) -> io::Result<()> {
	let metadata = fs::symlink_metadata(path)?;
	if !metadata.is_file() {
		return Err(unsupported_target(path));
	}

	let file = linux::open_for_wiping(path)?;
	if !file.metadata()?.is_file() {
		return Err(unsupported_target(path));
	}

	for pattern_spec in passes {
		let mut pattern = pattern_spec.create()?;
		overwrite(&file, metadata.len(), &mut *pattern)?;
		file.sync_all()?;
	}

	linux::obfuscate_metadata(&file)?;
	file.set_len(0)?;
	file.sync_all()?;
	drop(file);

	let mut rename_bytes = [0; RENAME_BYTES];
	let mut random = RandomPattern::new()?;
	random.fill(&mut rename_bytes)?;
	let renamed = linux::rename_for_removal(path, &linux::random_name(&rename_bytes))?;
	linux::remove_file(&renamed)?;
	linux::sync_directory(&renamed)
}

fn overwrite(file: &File, length: u64, pattern: &mut dyn Pattern) -> io::Result<()> {
	let mut buffer = vec![0; CHUNK_SIZE];
	let mut writer = file.try_clone()?;
	writer.seek(SeekFrom::Start(0))?;
	let mut remaining = length;
	while remaining > 0 {
		let size = remaining.min(buffer.len() as u64) as usize;
		pattern.fill(&mut buffer[..size])?;
		writer.write_all(&buffer[..size])?;
		remaining -= size as u64;
	}
	Ok(())
}

fn unsupported_target(path: &Path) -> io::Error {
	io::Error::new(
		io::ErrorKind::InvalidInput,
		format!(
			"unsupported target (regular files only): {}",
			path.display()
		),
	)
}
