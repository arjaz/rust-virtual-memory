use std::collections::HashMap;

// The size of one physical page
const PAGE_SIZE: usize = 4096;

// This is a virtual page
#[derive(Debug, Clone)]
pub struct VirtualPage {
    presence: bool,
    reference: bool,
    modification: bool,
    // physical page address
    paddress: usize,
}

impl VirtualPage {
    pub fn new() -> Self {
        VirtualPage {
            presence: false,
            reference: false,
            modification: false,
            paddress: VirtualPage::null(),
        }
    }

    pub fn null() -> usize {
        0
    }

    // Updates all necessary bits of the page
    pub fn alloc(&mut self, paddress: usize) {
        self.presence = true;
        self.paddress = paddress;
    }

    // Swap out the page
    pub fn swap(&mut self) {
        self.paddress = VirtualPage::null();
        self.presence = true;
    }

    pub fn write(&mut self) {
        self.reference = true;
        self.modification = true;
    }

    pub fn read(&mut self) {
        self.reference = true;
    }
}

// This is a physical page
#[derive(Debug, Clone)]
pub struct PhysicalPage {
    address: usize,
}

impl PhysicalPage {
    pub fn new(address: usize) -> Self {
        PhysicalPage { address }
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
    // Position of the last checked virtual page, resets on every new process being registered
    hand: usize,
}

impl MemoryManager {
    pub fn new(mem_size: usize) -> Self {
        // create list of physical pages
        let physical_pages = (0..mem_size / PAGE_SIZE)
            // Get addresses
            .map(|x| x * PAGE_SIZE)
            .map(PhysicalPage::new)
            .collect::<PhysicalMemory>();

        MemoryManager {
            processes_memory: HashMap::new(),
            free_physical_pages: physical_pages,
            busy_physical_pages: Vec::new(),
            hand: 0,
        }
    }

    pub fn tick(&mut self, id: usize) {
        let memory = self.processes_memory.get_mut(&id).unwrap();
        for page in memory.iter_mut() {
            page.reference = false;
            page.modification = false;
        }
    }

    pub fn register(&mut self, id: usize, mem_size: usize) {
        assert!(
            !self.processes_memory.contains_key(&id),
            "This process is already registered"
        );

        self.hand = 0;

        let virtual_memory = (0..mem_size)
            // Get addresses
            .map(|_| VirtualPage::new())
            .collect::<VirtualMemory>();

        self.processes_memory.insert(id, virtual_memory);
    }

    pub fn write(&mut self, id: usize, address: usize) {
        assert!(
            self.processes_memory.contains_key(&id),
            "The process is not managed by this memory manager."
        );

        // Find the virtual page associated with this address
        let page_pos = address / 4096;
        let paddress = self.processes_memory.get_mut(&id).unwrap()[page_pos].paddress;

        let null_address = VirtualPage::null();
        if paddress == null_address {
            self.allocate(id, page_pos * 4096);
            let page = self
                .processes_memory
                .get_mut(&id)
                .unwrap()
                .get_mut(page_pos)
                .unwrap();
            page.write();
        } else {
            let page = self
                .processes_memory
                .get_mut(&id)
                .unwrap()
                .get_mut(page_pos)
                .unwrap();
            page.write();
        }
    }

    pub fn read(&mut self, id: usize, address: usize) {
        assert!(
            self.processes_memory.contains_key(&id),
            "The process is not managed by this memory manager."
        );

        // Find the virtual page associated with this address
        let page_pos = address / 4096;
        let paddress = self.processes_memory.get_mut(&id).unwrap()[page_pos].paddress;

        let null_address = VirtualPage::null();
        if paddress == null_address {
            self.allocate(id, page_pos * 4096);
            let page = self
                .processes_memory
                .get_mut(&id)
                .unwrap()
                .get_mut(page_pos)
                .unwrap();
            page.read();
        } else {
            let page = self
                .processes_memory
                .get_mut(&id)
                .unwrap()
                .get_mut(page_pos)
                .unwrap();
            page.read();
        }
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
                virtual_memory.len() >= (address / 4096),
                format!(
                    "Virtual memory doesn't contain that page[{:#010x}]",
                    address
                )
            );
        }

        let page_pos = address / 4096;
        // Try to map the physical page with the same address
        if let Some(position) = self
            .free_physical_pages
            .iter()
            .position(|p| p.address == address)
        {
            let virtual_memory = self.processes_memory.get_mut(&process_id).unwrap();
            let virtual_page = virtual_memory.get_mut(page_pos).unwrap();
            // NOTE: That's O(1), but doesn't preserve the ordering
            let physical_page = self.free_physical_pages.swap_remove(position);
            virtual_page.alloc(physical_page.address);
            println!("Page[{:#010x}] allocated", physical_page.address);
            self.busy_physical_pages.push(physical_page);
        } else if !self.free_physical_pages.is_empty() {
            // If there is no free physical page with the same address
            // map to the next free physical page
            let virtual_memory = self.processes_memory.get_mut(&process_id).unwrap();
            let virtual_page = virtual_memory.get_mut(page_pos).unwrap();

            let physical_page = self.free_physical_pages.swap_remove(0);
            virtual_page.alloc(physical_page.address);
            println!(
                "Page[{:#010x}] is busy, allocating page[{:#010x}]",
                address, physical_page.address
            );
            self.busy_physical_pages.push(physical_page);
        } else {
            // If there are no free physical pages left,
            // use clock algorithm and swap some pages
            'cycle: loop {
                // Keep track of the current virtual page number to store it as a new hand
                let mut current_page_num = 0;

                for virtual_memory in self.processes_memory.values_mut() {
                    for virtual_page in virtual_memory.iter_mut() {
                        // Skip until we get to the current virtual page
                        if self.hand != 0 {
                            self.hand -= 1;
                            current_page_num += 1;
                            continue;
                        }
                        // Check if the page should be swapped
                        match virtual_page.reference {
                            // Set the reference bit to false to indicate that the page has been inspected
                            true => virtual_page.reference = false,
                            // Swap the virtual page and allocate the physical one
                            false => {
                                let paddress = virtual_page.paddress;
                                virtual_page.swap();
                                virtual_memory[page_pos].alloc(paddress);
                                self.hand = current_page_num;
                                println!(
                                    "Page[{:#010x}] is busy, no free pages left, swapping out page[{:#010x}]",
                                    address, page_pos * 4096
                                );
                                break 'cycle;
                            }
                        }
                        current_page_num += 1;
                    }
                }
            }
        }
    }
}
