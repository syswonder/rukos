/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::sync::Arc;
use core::ffi::{c_char, c_int};

use axerrno::{LinuxError, LinuxResult};
use axio::{PollState, SeekFrom};
use axsync::Mutex;
use ruxfs::fops::OpenOptions;

use super::fd_ops::{get_file_like, FileLike};
use crate::{ctypes, utils::char_ptr_to_str};

pub struct File {
    inner: Mutex<ruxfs::fops::File>,
}

impl File {
    fn new(inner: ruxfs::fops::File) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    fn add_to_fd_table(self) -> LinuxResult<c_int> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl FileLike for File {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().read(buf)?)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().write(buf)?)
    }

    fn stat(&self) -> LinuxResult<ctypes::stat> {
        let metadata = self.inner.lock().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;
        Ok(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

/// Convert open flags to [`OpenOptions`].
fn flags_to_options(flags: c_int, _mode: ctypes::mode_t) -> OpenOptions {
    let flags = flags as u32;
    let mut options = OpenOptions::new();
    match flags & 0b11 {
        ctypes::O_RDONLY => options.read(true),
        ctypes::O_WRONLY => options.write(true),
        _ => {
            options.read(true);
            options.write(true);
        }
    };
    if flags & ctypes::O_APPEND != 0 {
        options.append(true);
    }
    if flags & ctypes::O_TRUNC != 0 {
        options.truncate(true);
    }
    if flags & ctypes::O_CREAT != 0 {
        options.create(true);
    }
    if flags & ctypes::O_EXEC != 0 {
        options.create_new(true);
    }
    options
}

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `EMFILE` if it already
/// has the maximum number of files open.
pub fn sys_open(filename: *const c_char, flags: c_int, mode: ctypes::mode_t) -> c_int {
    let filename = char_ptr_to_str(filename);
    debug!("sys_open <= {:?} {:#o} {:#o}", filename, flags, mode);
    syscall_body!(sys_open, {
        let options = flags_to_options(flags, mode);
        let file = ruxfs::fops::File::open(filename?, &options)?;
        File::new(file).add_to_fd_table()
    })
}

/// Open a file under a specific dir
///
/// TODO: Currently only support openat root directory
pub fn sys_openat(_fd: usize, path: *const c_char, flags: c_int, mode: ctypes::mode_t) -> c_int {
    let path = char_ptr_to_str(path);
    debug!("sys_openat <= {:?}, {:#o} {:#o}", path, flags, mode);
    syscall_body!(sys_openat, {
        let options = flags_to_options(flags, mode);
        let file = ruxfs::fops::File::open(path?, &options)?;
        File::new(file).add_to_fd_table()
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
pub fn sys_lseek(fd: c_int, offset: ctypes::off_t, whence: c_int) -> ctypes::off_t {
    debug!("sys_lseek <= {} {} {}", fd, offset, whence);
    syscall_body!(sys_lseek, {
        let pos = match whence {
            0 => SeekFrom::Start(offset as _),
            1 => SeekFrom::Current(offset as _),
            2 => SeekFrom::End(offset as _),
            _ => return Err(LinuxError::EINVAL),
        };
        let off = File::from_fd(fd)?.inner.lock().seek(pos)?;
        Ok(off)
    })
}

/// Synchronize a file's in-core state with storage device
///
/// TODO
pub unsafe fn sys_fsync(fd: c_int) -> c_int {
    debug!("sys_fsync <= fd: {}", fd);
    syscall_body!(sys_fsync, Ok(0))
}

/// Synchronize a file's in-core state with storage device
///
/// TODO
pub unsafe fn sys_fdatasync(fd: c_int) -> c_int {
    debug!("sys_fdatasync <= fd: {}", fd);
    syscall_body!(sys_fdatasync, Ok(0))
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_stat(path: *const c_char, buf: *mut ctypes::stat) -> c_int {
    let path = char_ptr_to_str(path);
    debug!("sys_stat <= {:?} {:#x}", path, buf as usize);
    syscall_body!(sys_stat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let mut options = OpenOptions::new();
        options.read(true);
        let file = ruxfs::fops::File::open(path?, &options)?;
        let st = File::new(file).stat()?;
        unsafe { *buf = st };
        Ok(0)
    })
}

/// retrieve information about the file pointed by `fd`
pub unsafe fn sys_fstat(fd: c_int, kst: *mut core::ffi::c_void) -> c_int {
    debug!("sys_fstat <= {} {:#x}", fd, kst as usize);
    syscall_body!(sys_fstat, {
        if kst.is_null() {
            return Err(LinuxError::EFAULT);
        }
        #[cfg(not(feature = "musl"))]
        {
            let buf = kst as *mut ctypes::stat;
            unsafe { *buf = get_file_like(fd)?.stat()? };
            Ok(0)
        }
        #[cfg(feature = "musl")]
        {
            let st = get_file_like(fd)?.stat()?;
            let kst = kst as *mut ctypes::kstat;
            unsafe {
                (*kst).st_dev = st.st_dev;
                (*kst).st_ino = st.st_dev;
                (*kst).st_mode = st.st_mode;
                (*kst).st_nlink = st.st_nlink;
                (*kst).st_uid = st.st_uid;
                (*kst).st_gid = st.st_gid;
                (*kst).st_size = st.st_size;
                (*kst).st_blocks = st.st_blocks;
                (*kst).st_blksize = st.st_blksize;
            }
            Ok(0)
        }
    })
}

/// Get the metadata of the symbolic link and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_lstat(path: *const c_char, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    let path = char_ptr_to_str(path);
    debug!("sys_lstat <= {:?} {:#x}", path, buf as usize);
    syscall_body!(sys_lstat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        unsafe { *buf = Default::default() }; // TODO
        Ok(0)
    })
}

/// `newfstatat` used by A64
pub unsafe fn sys_newfstatat(
    _fd: c_int,
    path: *const c_char,
    kst: *mut ctypes::kstat,
    flag: c_int,
) -> c_int {
    let path = char_ptr_to_str(path);
    debug!(
        "sys_newfstatat <= fd: {}, path: {:?}, flag: {:x}",
        _fd, path, flag
    );
    assert_eq!(_fd, ctypes::AT_FDCWD as c_int);
    syscall_body!(sys_newfstatat, {
        if kst.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let mut options = OpenOptions::new();
        options.read(true);
        let file = ruxfs::fops::File::open(path?, &options)?;
        let st = File::new(file).stat()?;
        unsafe {
            (*kst).st_dev = st.st_dev;
            (*kst).st_ino = st.st_dev;
            (*kst).st_mode = st.st_mode;
            (*kst).st_nlink = st.st_nlink;
            (*kst).st_uid = st.st_uid;
            (*kst).st_gid = st.st_gid;
            (*kst).st_size = st.st_size;
            (*kst).st_blocks = st.st_blocks;
            (*kst).st_blksize = st.st_blksize;
        }
        Ok(0)
    })
}

/// Get the path of the current directory.
pub fn sys_getcwd(buf: *mut c_char, size: usize) -> c_int {
    debug!("sys_getcwd <= {:#x} {}", buf as usize, size);
    syscall_body!(sys_getcwd, {
        if buf.is_null() {
            return Err(LinuxError::EINVAL);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size as _) };
        let cwd = ruxfs::api::current_dir()?;
        let cwd = cwd.as_bytes();
        if cwd.len() < size {
            dst[..cwd.len()].copy_from_slice(cwd);
            dst[cwd.len()] = 0;
            Ok(cwd.len() + 1)
        } else {
            Err(LinuxError::ERANGE)
        }
    })
}

/// Rename `old` to `new`
/// If new exists, it is first removed.
///
/// Return 0 if the operation succeeds, otherwise return -1.
pub fn sys_rename(old: *const c_char, new: *const c_char) -> c_int {
    syscall_body!(sys_rename, {
        let old_path = char_ptr_to_str(old)?;
        let new_path = char_ptr_to_str(new)?;
        debug!("sys_rename <= old: {:?}, new: {:?}", old_path, new_path);
        ruxfs::api::rename(old_path, new_path)?;
        Ok(0)
    })
}

/// Rename at certain directory pointed by `oldfd`
///
/// TODO: only support `oldfd`, `newfd` equals to AT_FDCWD
pub fn sys_renameat(oldfd: c_int, old: *const c_char, newfd: c_int, new: *const c_char) -> c_int {
    let old_path = char_ptr_to_str(old);
    let new_path = char_ptr_to_str(new);
    debug!(
        "sys_renameat <= oldfd: {}, old: {:?}, newfd: {}, new: {:?}",
        oldfd, old_path, newfd, new_path
    );
    assert_eq!(oldfd, ctypes::AT_FDCWD as c_int);
    assert_eq!(newfd, ctypes::AT_FDCWD as c_int);
    syscall_body!(sys_renameat, {
        ruxfs::api::rename(old_path?, new_path?)?;
        Ok(0)
    })
}

/// Remove a directory, which must be empty
pub fn sys_rmdir(pathname: *const c_char) -> c_int {
    syscall_body!(sys_rmdir, {
        let path = char_ptr_to_str(pathname)?;
        debug!("sys_rmdir <= path: {:?}", path);
        ruxfs::api::remove_dir(path)?;
        Ok(0)
    })
}

/// Removes a file from the filesystem.
pub fn sys_unlink(pathname: *const c_char) -> c_int {
    syscall_body!(sys_unlink, {
        let path = char_ptr_to_str(pathname)?;
        debug!("sys_unlink <= path: {:?}", path);
        ruxfs::api::remove_file(path)?;
        Ok(0)
    })
}

/// deletes a name from the filesystem
pub fn sys_unlinkat(fd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    debug!(
        "sys_unlinkat <= fd: {}, pathname: {:?}, flags: {}",
        fd,
        char_ptr_to_str(pathname),
        flags
    );
    if flags as u32 & ctypes::AT_REMOVEDIR != 0 {
        return sys_rmdir(pathname);
    }
    sys_unlink(pathname)
}

/// Creates a new, empty directory at the provided path.
pub fn sys_mkdir(pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    // TODO: implement mode
    syscall_body!(sys_mkdir, {
        let path = char_ptr_to_str(pathname)?;
        debug!("sys_mkdir <= path: {:?}, mode: {:?}", path, mode);
        ruxfs::api::create_dir(path)?;
        Ok(0)
    })
}

/// attempts to create a directory named pathname under directory pointed by `fd`
///
/// TODO: currently fd is not used
pub fn sys_mkdirat(fd: c_int, pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    debug!(
        "sys_mkdirat <= fd: {}, pathname: {:?}, mode: {:x?}",
        fd,
        char_ptr_to_str(pathname),
        mode
    );
    sys_mkdir(pathname, mode)
}

/// Changes the ownership of the file referred to by the open file descriptor fd
pub fn sys_fchownat(
    fd: c_int,
    path: *const c_char,
    uid: ctypes::uid_t,
    gid: ctypes::gid_t,
    flag: c_int,
) -> c_int {
    debug!(
        "sys_fchownat <= fd: {}, path: {:?}, uid: {}, gid: {}, flag: {}",
        fd,
        char_ptr_to_str(path),
        uid,
        gid,
        flag
    );
    syscall_body!(sys_fchownat, Ok(0))
}
