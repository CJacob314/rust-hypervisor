use std::fs::File;
use std::io::Read;
use std::num::NonZeroUsize;
use std::ops::{Add, BitAnd};
use std::{io, path::Path};
use std::os::fd::RawFd;

use nix::{fcntl::{open, OFlag}, sys::stat::Mode};
use nix::unistd::close;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::MmapChunk;

pub struct Kvm {
    fd: RawFd,
    vm_fd: RawFd,
    vcpu_fd: RawFd,
    memory: MmapChunk,
}

impl Kvm {
    pub fn new(binary_path: impl AsRef<Path>) -> io::Result<Self> {
        let mut binary_file = File::open(binary_path)?;
        let binary_file_size = binary_file.metadata()?.len();

        let fd = open(KVM_DEVICE_FILE_PATH, OFlag::O_RDWR, Mode::empty()).map_err(|errno| io::Error::from_raw_os_error(errno as _))?;

        if Self::api_version(fd) != 12 {
            Err(io::Error::new(io::ErrorKind::Unsupported, "Linux KVM version is not 12"))
        } else {
            let vm_fd = Self::create_vm(fd)?;

            let memory = MmapChunk::new(NonZeroUsize::new(binary_file_size as _).expect("binary_path file should have nonzero size"))?;
            binary_file.read(memory.as_slice())?; // Read bytes into mmap chunk
    
            let region = kvm_userspace_memory_region {
                slot: 0,
                guest_phys_addr: 0,
                memory_size: pgroundup(binary_file_size),
                userspace_addr: memory.ptr() as u64,
                flags: 0,
            };

            // Set memory region
            if unsafe { libc::ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &raw const region) } < 0 {
                return Err(io::Error::last_os_error());
            }

            let vcpu_fd = Self::create_vcpu(vm_fd)?;

            let mut regs: kvm_regs = Default::default();
            let mut sregs: kvm_sregs = Default::default();

            // Get sregs
            if unsafe { libc::ioctl(vcpu_fd, KVM_GET_SREGS, &raw mut sregs) } < 0 {
                return Err(io::Error::last_os_error());
            }

            // Modify sregs
            sregs.cs.base = 0;
            sregs.cs.selector = 0;

            // Set the modified sregs
            if unsafe { libc::ioctl(vcpu_fd, KVM_SET_SREGS, &raw mut sregs) } < 0 {
                return Err(io::Error::last_os_error());
            }

            // Get regs
            if unsafe { libc::ioctl(vcpu_fd, KVM_GET_REGS, &raw mut regs) } < 0 {
                return Err(io::Error::last_os_error());
            }

            regs.rip = 0x0;
            regs.rflags = 0x2; // Minimum flags required
                               
            if unsafe { libc::ioctl(vcpu_fd, KVM_SET_REGS, &raw mut regs) } < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(Self { fd, vm_fd, vcpu_fd, memory })
        }
    }

    fn api_version(fd: RawFd) -> i32 {
        unsafe { libc::ioctl(fd, KVM_GET_API_VERSION, 0) }
    }

    fn create_vm(fd: RawFd) -> io::Result<i32> {
        let fd = unsafe { libc::ioctl(fd, KVM_CREATE_VM, 0) };

        if fd < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(fd)
        }
    }

    fn create_vcpu(fd: RawFd) -> io::Result<RawFd> {
        let ioctl_return = unsafe { libc::ioctl(fd, KVM_CREATE_VCPU, 0) };
        if ioctl_return < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ioctl_return)
        }
    }
}

impl Drop for Kvm {
    fn drop(&mut self) {
        close(self.vm_fd).expect("nix::unistd::close should not fail on KVM fd");
        close(self.fd).expect("nix::unistd::close should not fail on KVM file fd");
    }
}

fn pgroundup<T: Add<Output = T> + BitAnd<Output = T> + From<u32>>(value: T) -> T {
    (value + 4095.into()) & (!4095).into()
}

const KVM_DEVICE_FILE_PATH: &'static str = "/dev/kvm";

