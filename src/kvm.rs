use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::num::NonZeroUsize;
use std::ops::{Add, BitAnd};
use std::{io, path::Path};
use std::os::fd::{BorrowedFd, RawFd};

use nix::{fcntl::{open, OFlag}, sys::stat::Mode};
use nix::unistd::close;

use crate::error;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::MmapChunk;

pub struct Kvm {
    fd: RawFd,
    vm_fd: RawFd,
    vcpu_fd: RawFd,
    memory: MmapChunk,
    actions: BTreeMap<u32, Box<dyn Fn(&Kvm) -> bool>>,
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

            let memory = MmapChunk::new_anonymous(NonZeroUsize::new(binary_file_size as _).expect("binary_path file should have nonzero size"))?;
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

            Ok(Self {
                fd, vm_fd, vcpu_fd, memory, 
                actions: BTreeMap::new() 
            })
        }
    }

    pub fn register_action(&mut self, event: u32, action: Box<dyn Fn(&Kvm) -> bool>) {
        self.actions.insert(event, action);
    }

    pub fn run(&self) -> error::Result<()> {
        let run_size: isize = unsafe { libc::ioctl(self.fd, KVM_GET_VCPU_MMAP_SIZE, 0) } as _;
        if run_size < 0 {
            return Err(io::Error::last_os_error().into());
        }

        let run_map_size = NonZeroUsize::new(run_size as _).ok_or(error::Error::ZeroSizedKvmSharedMemoryRegion)?;
        let borrowed_vcpu_fd = unsafe { BorrowedFd::borrow_raw(self.vcpu_fd) };
        let run_map = MmapChunk::new(run_map_size, borrowed_vcpu_fd)?;

        let run = run_map.ptr() as *mut kvm_run;
        loop {
            if unsafe { libc::ioctl(self.vcpu_fd, KVM_RUN, 0) } < 0 {
                return Err(io::Error::last_os_error().into());
            }

            let exit_reason = unsafe { std::ptr::read_volatile(&(*run).exit_reason) };

            let action: &Box<dyn for<'a> Fn(&'a Kvm) -> bool> = self.actions.get(&exit_reason).ok_or(error::Error::NoActionRegistered(exit_reason))?;
            if !action(self) {
                break;
            }
        }

        Ok(())
    }

    pub fn api_version(fd: RawFd) -> i32 {
        unsafe { libc::ioctl(fd, KVM_GET_API_VERSION, 0) }
    }

    pub fn regs(&self) -> io::Result<kvm_regs> {
        let mut regs: kvm_regs = Default::default();

        if unsafe { libc::ioctl(self.vcpu_fd, KVM_GET_REGS, &raw mut regs) } < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(regs)
        }
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
        close(self.vcpu_fd).expect("nix::unistd::close should not fail on KVM vCPU fd");
    }
}

fn pgroundup<T: Add<Output = T> + BitAnd<Output = T> + From<u32>>(value: T) -> T {
    (value + 4095.into()) & (!4095).into()
}

const KVM_DEVICE_FILE_PATH: &str = "/dev/kvm";

