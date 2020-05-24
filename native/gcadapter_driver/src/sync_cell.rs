use std::mem::{MaybeUninit, transmute_copy};

use std::marker::Send;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Size of the buffer used for the underlying circular buffer.
const BUFFER_SIZE: usize = 32;

/// An unsafe, lockless cell which is syncronized across threads.
///
/// When a reader requests the data held by this cell, the most recently
/// written data will be given to the reader.
pub struct SyncCell<T: Default> {
    control_block: Box<ControlBlock<T>>
}

impl<T: Default> SyncCell<T> {
    pub fn new() -> SyncCell<T> {
        SyncCell {
            control_block: Box::new(ControlBlock::new())
        }
    }

    pub fn read(&self) -> &T {
        unsafe { self.control_block.read() }
    }

    pub fn create_writer(&self) -> SyncCellWriter<T> {
        SyncCellWriter {
            control_block: unsafe { transmute_copy(&self.control_block) }
        }
    }
}


/// Control block for a SyncCell. Stored in the heap and used for
/// resource reclamation.
struct ControlBlock<T: Default> {
    current: usize,

    last_update: AtomicPtr<T>,
    buffer: [T; BUFFER_SIZE]
}

impl<T: Default> ControlBlock<T> {
    fn new() -> ControlBlock<T> {
        ControlBlock {
            current: 0,

            last_update: AtomicPtr::new(std::ptr::null_mut::<T>()),
            buffer: Default::default(),
        }
    }

    pub fn update(&mut self, data: T) {
        self.current = (self.current + 1) % BUFFER_SIZE;
        self.buffer[self.current] = data;

        unsafe {
            self.last_update = self.buffer.as_mut_ptr().add(self.current).into();
        }
    }

    pub fn read(&self) -> &T {
        unsafe {
            &*self.last_update.load(Ordering::Relaxed)
        }
    }
}

/// Writes into a SyncCell.
pub struct SyncCellWriter<T: Default> {
    control_block: *mut ControlBlock<T>
}

unsafe impl<T: Default> Send for SyncCellWriter<T> {}

impl<T: Default> SyncCellWriter<T> {
    pub fn write(&self, data: T) {
        unsafe {
            (&mut *self.control_block).update(data)
        }
    }
}