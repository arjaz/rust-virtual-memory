// The size of one physical page
const PAGE_SIZE: usize = 4096;

// This is a virtual page
#[derive(Debug, Clone)]
pub struct VirtualPage {
    presence: bool,
    reference: bool,
    modification: bool,
    // Address the process would see
    address: usize,
    // physical page address
    paddress: Option<usize>,
}

impl VirtualPage {
    pub fn new(address: usize) -> Self {
        VirtualPage {
            presence: false,
            reference: false,
            modification: false,
            address,
            paddress: None,
        }
    }
}

// This is a physical page
#[derive(Debug, Clone)]
pub struct PhysicalPage {
    address: usize,
    // That should always be 4K
    size: usize,
}

impl PhysicalPage {
    pub fn new(address: usize) -> Self {
        PhysicalPage {
            address,
            size: PAGE_SIZE,
        }
    }
}

pub type VirtualMemory = Vec<VirtualPage>;
pub type PhysicalMemory = Vec<PhysicalPage>;

// This is a memory manager
#[derive(Debug)]
pub struct MemoryManager {
    // virtual_pages: VirtualMemory,
    free_physical_pages: PhysicalMemory,
    busy_physical_pages: PhysicalMemory,
}

impl MemoryManager {
    pub fn new(mem_size: usize) -> Self {
        // create list of physical pages
        let physical_pages = (0..mem_size)
            .map(|x| x * PAGE_SIZE)
            .map(PhysicalPage::new)
            .collect::<PhysicalMemory>();

        MemoryManager {
            free_physical_pages: physical_pages,
            busy_physical_pages: Vec::new(),
        }
    }

    // Try to map a virtual page with the specified address to a free physical page
    pub fn allocate(&mut self, virtual_memory: &mut VirtualMemory, address: usize) {
        assert!(
            virtual_memory
                .iter()
                .find(|p| p.address == address)
                .is_some(),
            "Virtual memory doesn't contain a page with the specified address"
        );
        let virtual_page = virtual_memory
            .iter_mut()
            .find(|p| p.address == address)
            .unwrap();

        // Try to map the physical page with the same address
        if let Some(position) = self
            .free_physical_pages
            .iter()
            .position(|p| p.address == address)
        {
            // NOTE: That's O(1), but doesn't preserve the ordering
            let physical_page = self.free_physical_pages.swap_remove(position);
            virtual_page.paddress = Some(physical_page.address);
            self.busy_physical_pages.push(physical_page);
        } else if !self.free_physical_pages.is_empty() {
            // If there is no free physical page with the same address
            // map to the next free physical page
            let physical_page = self.free_physical_pages.swap_remove(0);
            virtual_page.paddress = Some(physical_page.address);
            self.busy_physical_pages.push(physical_page);
        } else {
            // If there is no free physical page left,
            // use clock algorithm and swap some pages
        }

        println!("Allocating virtual memory and mapping it to physical memory")
    }
}
