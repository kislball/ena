use crate::{
    blocks::Blocks,
    machine::{self, VMError},
};
use flexstr::SharedStr;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize, PartialEq)]
pub enum ThreadSupervisorError {
    #[error("unknown thread - `{0}`")]
    UnknownThread(u32),
}

#[derive(Default)]
pub struct ThreadSupervisor {
    threads: HashMap<u32, machine::VM>,
    increment: u32,
}

impl ThreadSupervisor {
    pub fn send_message(
        &mut self,
        value: enalang_ir::Value,
        to: u32,
        from: u32,
    ) -> Result<(), ThreadSupervisorError> {
        let vm = self.threads.get_mut(&to);
        let vm = match vm {
            Some(i) => i,
            None => return Err(ThreadSupervisorError::UnknownThread(to)),
        };

        vm.message_stack
            .as_mut()
            .expect("vm in supervisor to have message stack initialized")
            .push(machine::Message {
                from,
                content: value,
            });

        Ok(())
    }

    pub fn get_thread(&self, id: u32) -> Option<&machine::VM> {
        self.threads.get(&id)
    }

    pub fn get_thread_mut(&mut self, id: u32) -> Option<&mut machine::VM> {
        self.threads.get_mut(&id)
    }

    pub fn run_blocking(
        &mut self,
        id: u32,
        block: SharedStr,
        blocks: Blocks,
    ) -> Result<(), VMError> {
        let vm_thread = self.get_thread_mut(id).ok_or(VMError::SupervisorError(
            ThreadSupervisorError::UnknownThread(id),
        ))?;
        vm_thread.run(&block, blocks).map(|_| ())
    }

    pub fn supervise(
        &mut self,
        mut vm: machine::VM,
        supervisor: Arc<Mutex<ThreadSupervisor>>,
    ) -> Result<u32, ThreadSupervisorError> {
        let increment = self.get_increment();
        vm.thread_id = Some(increment);
        vm.message_stack = Some(Vec::new());
        vm.supervised_by = Some(supervisor);

        self.threads.insert(increment, vm);
        Ok(increment)
    }

    fn get_increment(&mut self) -> u32 {
        self.increment += 1;
        self.increment
    }
}

impl ThreadSupervisor {
    pub fn new() -> Self {
        Self::default()
    }
}
