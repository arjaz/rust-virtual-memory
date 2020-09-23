use std::collections::HashMap;

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
    swapped: bool,
}

impl VirtualPage {
    pub fn new(address: usize) -> Self {
        VirtualPage {
            presence: false,
            reference: false,
            modification: false,
            address,
            paddress: None,
            swapped: false,
        }
    }

    // Updates all necessary bits of the page
    pub fn alloc(&mut self, paddress: usize) {
        self.swapped = false;
        self.reference = true;
        self.presence = true;
        self.paddress = Some(paddress);
    }

    // Swap out the page
    pub fn swap(&mut self) {
        self.swapped = true;
        self.paddress = None;
        self.reference = false;
        self.presence = false;
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
    // HashMap from id of process to its memory
    processes_memory: HashMap<usize, VirtualMemory>,
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
            processes_memory: HashMap::new(),
            free_physical_pages: physical_pages,
            busy_physical_pages: Vec::new(),
        }
    }

    pub fn register(&mut self, id: usize, mem_size: usize) {
        assert!(
            !self.processes_memory.contains_key(&id),
            "This process is already registered"
        );

        let virtual_memory = (0..mem_size)
            .map(|x| x * 1024 * 4)
            .map(VirtualPage::new)
            .collect::<VirtualMemory>();

        self.processes_memory.insert(id, virtual_memory);
    }

    // Try to map a virtual page with the specified address to a free physical page
    pub fn allocate(&mut self, process_id: usize, address: usize) {
        {
            assert!(
                self.processes_memory.contains_key(&process_id),
                "The process is not managed by this memory manager."
            );

            let virtual_memory = self.processes_memory.get_mut(&process_id).unwrap();

            assert!(
                virtual_memory.iter().any(|p| p.address == address),
                "Virtual memory doesn't contain a page with the specified address"
            );
        }

        // Try to map the physical page with the same address
        if let Some(position) = self
            .free_physical_pages
            .iter()
            .position(|p| p.address == address)
        {
            let virtual_memory = self.processes_memory.get_mut(&process_id).unwrap();
            let virtual_page = virtual_memory
                .iter_mut()
                .find(|p| p.address == address)
                .unwrap();
            // NOTE: That's O(1), but doesn't preserve the ordering
            let physical_page = self.free_physical_pages.swap_remove(position);
            virtual_page.alloc(physical_page.address);
            self.busy_physical_pages.push(physical_page);
            println!("Mapping to the same addresss")
        } else if !self.free_physical_pages.is_empty() {
            // If there is no free physical page with the same address
            // map to the next free physical page
            let virtual_memory = self.processes_memory.get_mut(&process_id).unwrap();
            let virtual_page = virtual_memory
                .iter_mut()
                .find(|p| p.address == address)
                .unwrap();

            let physical_page = self.free_physical_pages.swap_remove(0);
            virtual_page.alloc(physical_page.address);
            self.busy_physical_pages.push(physical_page);
            println!("Mapping to a new addresss")
        } else {
            // If there are no free physical page left,
            // use clock algorithm and swap some pages
            let virtual_memory = self.processes_memory.get(&process_id).unwrap();
            let virtual_page_pos = virtual_memory
                .iter()
                .position(|p| p.address == address)
                .unwrap();
            'cycle: loop {
                for (id, virtual_memory) in self.processes_memory.iter_mut() {
                    for freed_virtual_page in
                        virtual_memory.iter_mut().filter(|p| p.paddress.is_some())
                    {
                        match freed_virtual_page.reference {
                            true => freed_virtual_page.reference = false,
                            false => {
                                let paddress = freed_virtual_page.paddress;
                                freed_virtual_page.swap();
                                virtual_memory[virtual_page_pos].alloc(paddress.unwrap());
                                println!(
                                    "Swapping out page {} of process {}",
                                    virtual_memory[virtual_page_pos].address, id
                                );
                                break 'cycle;
                            }
                        }
                    }
                }
            }
        }
    }
}
