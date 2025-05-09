use core::ffi::c_void;
use std::{io, num::NonZeroUsize, os::fd::AsFd, ptr::NonNull};

use nix::sys::mman::{mmap, mmap_anonymous, munmap, MapFlags, ProtFlags};

pub struct MmapChunk {
    ptr: NonNull<c_void>,
    size: NonZeroUsize,
}

impl MmapChunk {
    pub fn new_anonymous(size: NonZeroUsize) -> io::Result<Self> {
        let prot = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE;
        let flags = MapFlags::MAP_PRIVATE; // MAP_ANONYMOUS handled by `mmap_anonymous`

        let ptr = unsafe { mmap_anonymous(None, size, prot, flags) }.map_err(|errno| io::Error::from_raw_os_error(errno as _))?;

        Ok(Self{ ptr, size })
    }

    pub fn new<F: AsFd>(size: NonZeroUsize, fd: F) -> io::Result<Self> {
        let prot = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE;
        let flags = MapFlags::MAP_SHARED;

        let ptr = unsafe { mmap(None, size, prot, flags, fd, 0) }?;

        Ok(Self { ptr, size })
    }

    pub fn size(&self) -> usize {
        self.size.get()
    }

    pub fn ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    pub fn as_slice(&self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr() as _, self.size())
        }
    }
}

impl Drop for MmapChunk {
    fn drop(&mut self) {
        unsafe { munmap(self.ptr, self.size.into()) }.expect("MmapChunk drop munmap call failed");
    }
}
