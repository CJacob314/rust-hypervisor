mod kvm;
mod mmap_chunk;

use std::error::Error;

use kvm::Kvm;
pub use mmap_chunk::MmapChunk;

fn main() -> Result<(), Box<dyn Error>> {
    let kvm = Kvm::new("./guest.bin")?;
    Ok(())
}

const EXIT_FAILURE: i32 = 1;
