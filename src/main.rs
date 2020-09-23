use std::thread;
use std::time::Duration;

mod pages;
use pages::*;

#[derive(Debug)]
pub struct Process {
    id: usize,
    // The time the process would live
    lifetime: usize,
    // List of addresses
    used_addresses: Vec<usize>,
    // The number of times the process would reference the memory
    memory_references: u32,
}

impl Process {
    fn new(id: usize, lifetime: usize, used_addresses: Vec<usize>, memory_references: u32) -> Self {
        Process {
            id,
            lifetime,
            used_addresses,
            memory_references,
        }
    }
}

// Each process has its own (virtual) memory space After that it asks
// the memory manager to allocate some memory And the memory manager
// will check the physical space and provide a mapping from virtual to
// physical memory space

fn main() {
    // Set maximal number of processes
    const MAX_PROCESSES: usize = 4;

    // Set time of creation of each process
    let mut creation_times = (0..MAX_PROCESSES).collect::<Vec<usize>>();

    // Initialize the memory manager
    const MEM_SIZE: usize = 4;
    let mut memory_manager = MemoryManager::new(MEM_SIZE);

    let mut processes: Vec<Process> = Vec::new();
    // The system lifetime
    let mut current_tick = 0;

    let mut id = 0;
    while !processes.is_empty() || !creation_times.is_empty() {
        println!("Tick: {}", current_tick);

        // Initialize a vector which stores the indeces of the creation_times to be deleted
        let mut to_delete: Vec<usize> = Vec::new();

        // Go over the creation times and see if a new process should be created
        for (index, creation_time) in creation_times.iter().enumerate() {
            // Create a new process if the time matches
            if current_tick == *creation_time {
                to_delete.push(index);

                let lifetime = 2;
                let used_addresses = vec![0x00000000];
                let memory_references = 2;
                id += 1;

                let process = Process::new(id, lifetime, used_addresses, memory_references);
                memory_manager.register(process.id, MEM_SIZE);
                processes.push(process);
                println!(
                    "A process with lifetime {} is created and registered",
                    processes.last().unwrap().lifetime
                );
            }
        }

        // Delete times of the already created processes
        for index in to_delete {
            creation_times.remove(index);
        }

        let mut to_delete: Vec<usize> = Vec::new();
        for (position, process) in processes
            .iter_mut()
            .enumerate()
            .filter(|(_, p)| p.lifetime > 0)
        {
            println!("Process[{:?}] is acting.", position);

            // For now ask for your memory
            if let Some(address) = process.used_addresses.first() {
                memory_manager.allocate(process.id, *address)
            }

            process.lifetime -= 1;
            if process.lifetime == 0 {
                to_delete.push(position);
            }
        }

        for position in to_delete {
            processes.remove(position);
        }

        current_tick += 1;
        println!();
        thread::sleep(Duration::from_millis(1000));
    }

    // Upon creation of a process, create a new page table with demand paging
    // Create a list of pages each process would use, set number of times the process would reference the memory
}
