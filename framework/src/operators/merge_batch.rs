use super::act::Act;
use super::iterator::BatchIterator;
use super::packet_batch::PacketBatch;
use super::Batch;
use common::*;
use interface::{PacketTx, Pdu};
use scheduler::Executable;
use std::cmp;

pub struct MergeBatchTraitObj {
    parents: Vec<Box<dyn Batch>>,
    which: usize,
    slot: usize, // index into selector
    selector: Vec<usize>,
}

impl MergeBatchTraitObj {
    pub fn new(parents: Vec<Box<dyn Batch>>) -> MergeBatchTraitObj {
        let selector: Vec<usize> = (0..parents.len()).collect();
        MergeBatchTraitObj {
            parents,
            which: selector[0],
            slot: 0,
            selector,
        }
    }
    pub fn new_with_selector(parents: Vec<Box<dyn Batch>>, selector: Vec<usize>) -> MergeBatchTraitObj {
        MergeBatchTraitObj {
            parents,
            which: selector[0],
            slot: 0,
            selector,
        }
    }
}

impl Batch for MergeBatchTraitObj {
    #[inline]
    fn queued(&self) -> usize {
        let mut result = 0;
        for parent in &self.parents {
            result = parent.queued();
            if result > 0 {
                break;
            }
        }
        result
    }
}

impl BatchIterator for MergeBatchTraitObj {
    #[inline]
    fn start(&mut self) -> usize {
        self.parents[self.which].start()
    }

    #[inline]
    fn next_payload(&mut self, idx: usize) -> Option<Pdu> {
        self.parents[self.which].next_payload(idx)
    }
}

/// Internal interface for packets.
impl Act for MergeBatchTraitObj {
    #[inline]
    fn act(&mut self) -> (u32, i32) {
        self.parents[self.which].act()
    }

    #[inline]
    fn done(&mut self) {
        self.parents[self.which].done();
        let next = self.slot + 1;
        if next == self.selector.len() {
            self.slot = 0
        } else {
            self.slot = next
        }
        self.which = self.selector[self.slot]
    }

    #[inline]
    fn send_q(&mut self, port: &mut dyn PacketTx) -> errors::Result<u32> {
        self.parents[self.which].send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parents.iter().fold(0, |acc, x| cmp::max(acc, x.capacity()))
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.parents[self.which].drop_packets(idxes)
    }

    #[inline]
    fn drop_packets_all(&mut self) -> Option<usize> {
        self.parents[self.which].drop_packets_all()
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parents[self.which].clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self.parents[self.which].get_packet_batch()
    }
}

impl Executable for MergeBatchTraitObj {
    #[inline]
    fn execute(&mut self) -> (u32, i32) {
        let count = self.act();
        self.done();
        count
    }
}

pub struct MergeBatch<T: Batch> {
    parents: Vec<T>,
    which: usize,
    slot: usize, // index into selector
    selector: Vec<usize>,
}

impl<T: Batch> MergeBatch<T> {
    pub fn new(parents: Vec<T>) -> MergeBatch<T> {
        let selector: Vec<usize> = (0..parents.len()).collect();
        MergeBatch {
            parents,
            which: selector[0],
            slot: 0,
            selector,
        }
    }
    pub fn new_with_selector(parents: Vec<T>, selector: Vec<usize>) -> MergeBatch<T> {
        MergeBatch {
            parents,
            which: selector[0],
            slot: 0,
            selector,
        }
    }
}

impl<T: Batch> Batch for MergeBatch<T> {
    #[inline]
    fn queued(&self) -> usize {
        let mut result = 0;
        // TODO check if we should take the max length queue, or do we only go with MergeBatchAuto?
        for parent in &self.parents {
            result = parent.queued();
            if result > 0 {
                break;
            }
        }
        result
    }
}

impl<T: Batch> BatchIterator for MergeBatch<T> {
    #[inline]
    fn start(&mut self) -> usize {
        self.parents[self.which].start()
    }

    #[inline]
    fn next_payload(&mut self, idx: usize) -> Option<Pdu> {
        self.parents[self.which].next_payload(idx)
    }
}

/// Internal interface for packets.
impl<T: Batch> Act for MergeBatch<T> {
    #[inline]
    fn act(&mut self) -> (u32, i32) {
        self.parents[self.which].act()
    }

    #[inline]
    fn done(&mut self) {
        self.parents[self.which].done();
        let next = self.slot + 1;
        if next == self.selector.len() {
            self.slot = 0
        } else {
            self.slot = next
        }
        self.which = self.selector[self.slot]
    }

    #[inline]
    fn send_q(&mut self, port: &mut dyn PacketTx) -> errors::Result<u32> {
        self.parents[self.which].send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parents.iter().fold(0, |acc, x| cmp::max(acc, x.capacity()))
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.parents[self.which].drop_packets(idxes)
    }

    #[inline]
    fn drop_packets_all(&mut self) -> Option<usize> {
        self.parents[self.which].drop_packets_all()
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parents[self.which].clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        self.parents[self.which].get_packet_batch()
    }

    //    #[inline]
    ////    fn get_task_dependencies(&self) -> Vec<usize> {
    //        let mut deps = Vec::with_capacity(self.parents.len()); // Might actually need to be larger, will get resized
    //        for parent in &self.parents {
    //            deps.extend(parent.get_task_dependencies().iter())
    //        }
    //        // We need to eliminate duplicate tasks. Fortunately this is not called on the critical path so it is fine to do
    //        // it this way.
    //        deps.sort();
    //        deps.dedup();
    //        deps
    //    }
}

impl<T: Batch> Executable for MergeBatch<T> {
    #[inline]
    fn execute(&mut self) -> (u32, i32) {
        let count = self.act();
        self.done();
        count
    }

    //    #[inline]
    //    fn dependencies(&mut self) -> Vec<usize> {
    //        self.get_task_dependencies()
    //    }
}
