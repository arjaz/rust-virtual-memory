#[derive(Debug)]
pub struct Process {
    pub id: usize,
    // The time the process would live.
    // Also, number of times the porcess would reference the memory
    pub lifetime: usize,
    // List of adresses
    pub used_addresses: Vec<usize>,
}

impl Process {
    pub fn new(
        id: usize,
        max_lifetime: usize,
        max_address: usize,
        max_addresses_num: usize,
    ) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let lifetime = rng.gen_range(1, max_lifetime + 1);
        let used_addresses = (0..rng.gen_range(1, max_addresses_num + 1))
            .map(|_| rng.gen_range(1, max_address / 4096) * 4096)
            .collect::<Vec<usize>>();
        Process {
            id,
            lifetime,
            used_addresses,
        }
    }
}
