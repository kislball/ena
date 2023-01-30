use crate::vm::ir;
use std::collections::HashMap;

use super::machine;

#[derive(Debug)]
pub enum HeapError {
    BlockNotAllocated(usize),
    BadPointer(usize),
}

pub fn heap_result_into_vm<T>(r: Result<T, HeapError>) -> Result<T, machine::VMError> {
    r.map_err(machine::VMError::HeapError)
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryBlock {
    pub pointer: usize,
    pub size: usize,
}

pub struct Heap {
    pub gc_enabled: bool,
    pub debug_gc: bool,
    heap: HashMap<usize, ir::Value>,
    blocks: Vec<MemoryBlock>,
    rc: HashMap<usize, usize>,
}

impl Heap {
    pub fn new(gc_enabled: bool, debug_gc: bool) -> Self {
        Self {
            heap: HashMap::new(),
            rc: HashMap::new(),
            blocks: vec![],
            gc_enabled,
            debug_gc,
        }
    }

    fn find_space(&self, size: usize) -> usize {
        let mut pointer = 0;

        'top: loop {
            if size == 1 {
                if self.is_used(pointer) {
                    pointer += 1;
                    continue;
                } else {
                    break pointer;
                }
            }
            for addr in pointer..(pointer + (size - 1)) {
                if self.is_used(addr) {
                    pointer += 1;
                    continue 'top;
                }
            }
            break pointer;
        }
    }

    fn get_pointer_owner_block(&self, pointer: usize) -> Option<MemoryBlock> {
        for block in &self.blocks {
            if pointer == block.pointer
                || (pointer > block.pointer && pointer < block.pointer + block.size)
            {
                return Some(*block);
            }
        }

        None
    }

    fn is_used(&self, pointer: usize) -> bool {
        self.get_pointer_owner_block(pointer).is_some()
    }

    fn create_block(&mut self, size: usize) -> MemoryBlock {
        let block = MemoryBlock {
            size,
            pointer: self.find_space(size),
        };

        self.blocks.push(block);

        block
    }

    fn get_block(&mut self, pointer: usize) -> Option<MemoryBlock> {
        for block in &self.blocks {
            if block.pointer == pointer {
                return Some(*block);
            }
        }

        None
    }

    fn move_memory(&mut self, src: usize, dest: usize, size: usize) {
        for i in 0..size {
            if let Some(val) = self.heap.get(&(src + i)) {
                           self.heap.insert(dest + i, val.clone());
                 self.heap.remove(&(src + i));
               }
        }
    }

    fn clear_memory(&mut self, pointer: usize, size: usize) -> Result<(), HeapError> {
        if self.debug_gc {
            println!("GC_DEBUG: heap size before cleaning: {}", self.heap.len());
        }

        for i in 0..size {
            if self.debug_gc && self.heap.contains_key(&(pointer + i)) {
                println!("GC_DEBUG: freeing pointer {}", pointer + i);
            }
            if let Some(ir::Value::Pointer(i)) = self.heap.remove(&(pointer + i)) {
                             self.rc_minus(i)?;
                 }
        }

        if self.debug_gc {
            println!(
                "GC_DEBUG: new heap size after cleaning: {}",
                self.heap.len()
            );
        }

        Ok(())
    }

    fn remove_block(&mut self, pointer: usize) {
        self.blocks.retain(|x| x.pointer != pointer);
    }

    fn rc_check(&mut self, pointer: usize) -> Result<(), HeapError> {
        if !self.gc_enabled {
            return Ok(());
        }

        let block = match self.get_block(pointer) {
            Some(i) => i,
            None => {
                return Err(HeapError::BlockNotAllocated(pointer));
            }
        };

        if let Some(i) = &self.rc.get(&block.pointer) {
            if self.debug_gc {
                println!(
                    "GC_DEBUG: checking pointer {} with RC: {}",
                    block.pointer, i
                );
            }
            if **i == 0 {
                if self.debug_gc {
                    println!(
                        "GC_DEBUG: freeing {} with size {}",
                        block.pointer, block.size
                    );
                }
                self.free(block.pointer)
            } else {
                Ok(())
            }
        } else {
            Err(HeapError::BadPointer(pointer))
        }
    }

    fn rc_change(&mut self, pointer: usize, plus: bool) -> Result<(), HeapError> {
        if !self.gc_enabled {
            return Ok(());
        }

        let block = match self.get_pointer_owner_block(pointer) {
            Some(i) => i,
            None => {
                return Err(HeapError::BlockNotAllocated(pointer));
            }
        };

        let new_value: usize = if plus {
            match &self.rc.get(&block.pointer) {
                Some(i) => *i + 1,
                None => 1,
            }
        } else {
            match &self.rc.get(&block.pointer) {
                Some(i) => *i - 1,
                None => 0,
            }
        };

        self.rc.insert(block.pointer, new_value);

        if self.debug_gc {
            println!(
                "GC_DEBUG: {}(in block {}) - RC:{}",
                pointer, block.pointer, new_value
            );
        }

        self.rc_check(block.pointer)
    }

    pub fn rc_reset(&mut self, pointer: usize) {
        let block = self.get_pointer_owner_block(pointer);
        if let Some(i) = block {
            self.rc.insert(i.pointer, 0);
        }
    }

    pub fn rc_minus(&mut self, pointer: usize) -> Result<(), HeapError> {
        self.rc_change(pointer, false)
    }

    pub fn rc_plus(&mut self, pointer: usize) -> Result<(), HeapError> {
        self.rc_change(pointer, true)
    }

    pub fn get(&self, pointer: usize) -> Option<ir::Value> {
        if self.gc_enabled && !self.is_used(pointer) {
            println!(
                "GC_DEBUG: read from an unallocated area at pointer {pointer}",
            );
        }
        self.heap.get(&pointer).cloned()
    }

    pub fn set(&mut self, pointer: usize, value: ir::Value) -> Result<(), HeapError> {
        if self.gc_enabled && !self.is_used(pointer) {
            println!(
                "GC_DEBUG: write to an unallocated area at pointer {pointer}",
            );
        }
        self.heap.insert(pointer, value.clone());

        if let ir::Value::Pointer(val) = value {
            self.rc_plus(val)?;
        }
        Ok(())
    }

    pub fn realloc(&mut self, pointer: usize, size: usize) -> Result<usize, HeapError> {
        let block = self.get_block(pointer);
        let block = match block {
            Some(i) => i,
            None => {
                return Ok(self.alloc(size)?.pointer);
            }
        };

        if block.size >= size {
            return Ok(block.pointer);
        }

        let new_ptr = self.create_block(size).pointer;

        self.move_memory(pointer, new_ptr, block.size);
        self.remove_block(pointer);
        self.rc_reset(block.pointer);

        Ok(new_ptr)
    }

    pub fn alloc(&mut self, size: usize) -> Result<MemoryBlock, HeapError> {
        let block = self.create_block(size);

        self.rc_plus(block.pointer)?;

        Ok(block)
    }

    pub fn free(&mut self, pointer: usize) -> Result<(), HeapError> {
        let block = match self.get_block(pointer) {
            Some(i) => i,
            None => {
                return Ok(());
            }
        };

        self.clear_memory(block.pointer, block.size)?;
        self.remove_block(block.pointer);
        Ok(())
    }
}
