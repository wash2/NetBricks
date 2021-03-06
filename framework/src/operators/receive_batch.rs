use super::act::Act;
use super::iterator::*;
use super::packet_batch::PacketBatch;
use super::Batch;
use common::*;
use interface::{PacketRx, PacketTx, Pdu};

pub struct ReceiveBatch<T: PacketRx> {
    parent: PacketBatch,
    packet_rx: T,
    pub received: u64,
    urgent: bool,
}

impl<T: PacketRx> ReceiveBatch<T> {
    pub fn new_with_parent(parent: PacketBatch, packet_rx: T) -> ReceiveBatch<T> {
        ReceiveBatch {
            parent,
            packet_rx,
            received: 0,
            urgent: false,
        }
    }

    pub fn new(packet_rx: T) -> ReceiveBatch<T> {
        ReceiveBatch {
            parent: PacketBatch::new(32, false),
            packet_rx,
            received: 0,
            urgent: false,
        }
    }

    pub fn new_keep_mbuf(packet_rx: T) -> ReceiveBatch<T> {
        ReceiveBatch {
            parent: PacketBatch::new(32, true),
            packet_rx,
            received: 0,
            urgent: false,
        }
    }

    pub fn set_urgent(mut self) -> ReceiveBatch<T> {
        self.urgent = true;
        self
    }
}

impl<T: PacketRx> Batch for ReceiveBatch<T> {
    fn queued(&self) -> usize {
        if self.urgent {
            // we implement priority by faking the queue length
            if self.packet_rx.queued() > 0 {
                10000
            } else {
                0
            }
        } else {
            self.packet_rx.queued()
        }
    }
}

impl<T: PacketRx> BatchIterator for ReceiveBatch<T> {
    #[inline]
    fn start(&mut self) -> usize {
        self.parent.start()
    }

    #[inline]
    fn next_payload(&mut self, idx: usize) -> Option<Pdu> {
        self.parent.next_payload(idx)
    }
}

/// Internal interface for packets.
impl<T: PacketRx> Act for ReceiveBatch<T> {
    #[inline]
    fn act(&mut self) -> (u32, i32) {
        self.parent.act();
        self.parent
            .recv(&self.packet_rx)
            .and_then(|x| {
                self.received += x.0 as u64;
                Ok(x)
            })
            .expect("Receive failure")
    }

    #[inline]
    fn done(&mut self) {
        // Free up memory
        self.parent.deallocate_batch().expect("Deallocation failed");
    }

    #[inline]
    fn send_q(&mut self, port: &mut dyn PacketTx) -> errors::Result<u32> {
        self.parent.send_q(port)
    }

    #[inline]
    fn capacity(&self) -> i32 {
        self.parent.capacity()
    }

    #[inline]
    fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
        self.parent.drop_packets(idxes)
    }

    #[inline]
    fn drop_packets_all(&mut self) -> Option<usize> {
        self.parent.drop_packets_all()
    }

    #[inline]
    fn clear_packets(&mut self) {
        self.parent.clear_packets()
    }

    #[inline]
    fn get_packet_batch(&mut self) -> &mut PacketBatch {
        &mut self.parent
    }
}
