// This is a virtual page
#[derive(Debug)]
pub struct Page {
    presence: bool,
    reference: bool,
    modification: bool,
    // physical page number
    ppn: usize,
}

impl Page {
    pub fn new(ppn: usize) -> Self {
        Page {
            presence: false,
            reference: false,
            modification: false,
            ppn,
        }
    }
}
