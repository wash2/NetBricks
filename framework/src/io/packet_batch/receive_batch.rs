use super::act::Act;
use super::Batch;
use super::packet_batch::PacketBatch;
use super::iterator::BatchIterator;
use super::super::pmd::*;
use super::super::interface::Result;

// FIXME: Should we be handling multiple queues and ports here?
pub struct ReceiveBatch {
    parent: PacketBatch,
    port: PmdPort,
    queue: i32,
    pub received: u64,
}

impl ReceiveBatch {
    pub fn new_with_parent(parent: PacketBatch, port: PmdPort, queue: i32) -> ReceiveBatch {
        ReceiveBatch {
            parent: parent,
            port: port,
            queue: queue,
            received: 0,
        }
    }

    pub fn new(port: PmdPort, queue: i32) -> ReceiveBatch {
        ReceiveBatch {
            parent: PacketBatch::new(32),
            port: port,
            queue: queue,
            received: 0,
        }

    }
}

impl Batch for ReceiveBatch {
    type Parent = PacketBatch;

    #[inline]
    fn pop(&mut self) -> &mut PacketBatch {
        &mut self.parent
    }
}

impl BatchIterator for ReceiveBatch {
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_address(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn next_base_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        println!("recvbatch.next_address");
        self.parent.next_base_address(idx)
    }

    #[inline]
    unsafe fn next_base_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_base_payload(idx)
    }
}

/// Internal interface for packets.
impl Act for ReceiveBatch {
    #[inline]
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        self.parent
            .recv_queue(&mut self.port, self.queue)
            .and_then(|x| {
                self.received += x as u64;
                Ok(x)
            })
            .expect("Receive failed");
        self
    }

    #[inline]
    fn done(&mut self) -> &mut Self {
        // Free up memory
        self.parent.deallocate_batch().expect("Deallocation failed");
        self
    }

    #[inline]
    fn send_queue(&mut self, port: &mut PmdPort, queue: i32) -> Result<u32> {
        self.parent.send_queue(port, queue)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }
}
