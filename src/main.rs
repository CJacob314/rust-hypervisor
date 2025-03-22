mod kvm;
mod mmap_chunk;
mod error;

use kvm::{Kvm, KVM_EXIT_HLT};
pub use mmap_chunk::MmapChunk;

fn main() -> error::Result<()> {
    let mut kvm = Kvm::new("./guest.bin")?;
    
    println!("Initial register state:\n{:#?}", kvm.regs());

    kvm.register_action(KVM_EXIT_HLT, Box::new(|kvm| -> bool {
        println!("Register state at `hlt` instruction:\n{:#?}", kvm.regs());

        false // Tells the Kvm instance to not keep going.
    }));

    kvm.run()?;

    Ok(())
}

