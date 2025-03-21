use core::ffi::c_void;
use std::{num::NonZeroUsize, ptr::NonNull, io};

use nix::sys::mman::{mmap_anonymous, munmap, MapFlags, ProtFlags};

pub struct MmapChunk {
    ptr: NonNull<c_void>,
    size: NonZeroUsize,
}

impl MmapChunk {
    pub fn new(size: NonZeroUsize) -> io::Result<Self> {
        let prot = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE;
        let flags = MapFlags::MAP_PRIVATE; // MAP_ANONYMOUS handled by `mmap_anonymous`

        let ptr = unsafe { mmap_anonymous(None, size, prot, flags) }.map_err(|errno| io::Error::from_raw_os_error(errno as _))?;

        Ok(Self{ ptr, size })
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
