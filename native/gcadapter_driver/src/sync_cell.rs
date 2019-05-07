use std::mem::{MaybeUninit, transmute_copy};

use std::marker::Send;
use std::sync::atomic::AtomicPtr;

/// Size of the buffer used for the underlying circular buffer.
const BUFFER_SIZE: usize = 128;

/// An unsafe, lockless cell which is syncronized across threads.
///
/// When a reader requests the data held by this cell, the most recently
/// written data will be given to the reader.
pub struct SyncCell<T: Copy> {
    control_block: Box<ControlBlock<T>>
}

impl<T: Copy> SyncCell<T> {
    pub fn new() -> SyncCell<T> {
        SyncCell {
            control_block: Box::new(ControlBlock::new())
        }
    }

    pub fn create_reader(&self) -> SyncCellReader<T> {
        SyncCellReader {
            control_block: unsafe { transmute_copy(&self.control_block) }
        }
    }

    pub fn create_writer(&self) -> SyncCellWriter<T> {
        SyncCellWriter {
            control_block: unsafe { transmute_copy(&self.control_block) }
        }
    }
}


/// Control block for a SyncCell. Stored in the heap and used for
/// resource reclamation.
struct ControlBlock<T: Copy> {
    current: usize,

    last_update: AtomicPtr<T>,
    buffer: [T; BUFFER_SIZE]
}

impl<T: Copy> ControlBlock<T> {
    fn new() -> ControlBlock<T> {
        let mut buffer: [T; BUFFER_SIZE] = unsafe {
            [MaybeUninit::uninitialized().into_initialized(); BUFFER_SIZE]
        };

        ControlBlock {
            current: 0,

            last_update: buffer.as_mut_ptr().into(),
            buffer
        }
    }

    pub fn update(&mut self, data: T) {
        self.current = (self.current + 1) % BUFFER_SIZE;
        self.buffer[self.current] = data;

        unsafe {
            self.last_update = self.buffer.as_mut_ptr().add(self.current).into();
        }
    }

    pub fn read(&mut self) -> T {
        unsafe { **self.last_update.get_mut() }
    }
}

/// Writes into a SyncCell.
pub struct SyncCellWriter<T: Copy> {
    control_block: *mut ControlBlock<T>
}

unsafe impl<T: Copy> Send for SyncCellWriter<T> {}

impl<T: Copy> SyncCellWriter<T> {
    pub fn write(&self, data: T) {
        unsafe {
            (&mut *self.control_block).update(data)
        }
    }
}

/// Reads from a SyncCell.
pub struct SyncCellReader<T: Copy> {
    control_block: *mut ControlBlock<T>
}

unsafe impl<T: Copy> Send for SyncCellReader<T> {}

impl<T: Copy> SyncCellReader<T> {
    pub fn read(&self) -> T {
        unsafe {
            (&mut *self.control_block).read()
        }
    }
}
