use rand::Rng;
use std::thread;
use std::time::Duration;

mod pages;
use pages::*;

mod process;
use process::*;

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
    const PAGES: usize = 8;
    const MEM_SIZE: usize = (PAGES + 1) * 4096;
    let mut memory_manager = MemoryManager::new(MEM_SIZE);

    let mut processes: Vec<Process> = Vec::new();

    // The system lifetime
    let mut current_tick = 0;
    let mut id = 0;
    while !processes.is_empty() || !creation_times.is_empty() {
        println!("Tick: {}", current_tick);

        // Go over the creation times and see if a new process should be created
        for creation_time in creation_times.iter() {
            // Create a new process if the time matches
            if current_tick == *creation_time {
                let max_lifetime = 5;
                let max_address = MEM_SIZE;
                let max_adresses_num = 10;
                id += 1;

                let process = Process::new(id, max_lifetime, max_address, max_adresses_num);
                memory_manager.register(process.id, MEM_SIZE);
                processes.push(process);
                println!(
                    "A process[{}] with lifetime {} is created and registered",
                    processes.last().unwrap().id,
                    processes.last().unwrap().lifetime
                );
            }
        }
        creation_times.retain(|time| time > &current_tick);

        for process in processes.iter_mut().filter(|p| p.lifetime > 0) {
            memory_manager.tick(process.id);

            // Determine if it would be a random address or one from working memory
            let local = rand::random::<usize>() % 10 != 9;
            let mut rng = rand::thread_rng();
            if local {
                let page_pos = rng.gen_range(0, process.used_addresses.len());
                let address = process.used_addresses[page_pos];
                if rand::random::<bool>() {
                    println!("Process[{:?}] is writing to working memory.", process.id);
                    memory_manager.write(process.id, address);
                } else {
                    println!("Process[{:?}] is reading from working memory.", process.id);
                    memory_manager.read(process.id, address);
                }
            } else {
                let address = rng.gen_range(1, MEM_SIZE / 4) * 4;
                if rand::random::<bool>() {
                    println!(
                        "Process[{:?}] is writing to non-working memory.",
                        process.id
                    );
                    memory_manager.write(process.id, address);
                } else {
                    println!(
                        "Process[{:?}] is reading from non-working memory.",
                        process.id
                    );
                    memory_manager.read(process.id, address);
                }
            }

            process.lifetime -= 1;
        }
        processes.retain(|p| p.lifetime > 0);

        println!("Processes left: {}", processes.len());
        println!("Processes not spawned: {}", creation_times.len());

        current_tick += 1;
        println!();
        thread::sleep(Duration::from_millis(1000));
    }
}
