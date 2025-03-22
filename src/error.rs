use std::fmt::Debug;
use std::result;
use std::io;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("(IO) {0}")]
    IO(#[from] io::Error),

    #[error("KVM_GET_VCPU_MMAP_SIZE ioctl gave size of KVM shared memory region as 0")]
    ZeroSizedKvmSharedMemoryRegion,

    #[error("Kvm had no action registered for exit reason {0}")]
    NoActionRegistered(u32),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T> = result::Result<T, Error>;
