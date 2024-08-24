use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Child;
use std::process::Command;
use std::mem::size_of;

use crate::dwarf_data::DwarfData;

#[derive(Clone)]
pub struct Breakpoint {
    addr: usize,
    orig_byte: u8,
}

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
    pub breakpoints: HashMap<usize, Breakpoint>, // Mapping breakpoint addresses to Breakpoint structs
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints: &Vec<usize>) -> Option<Inferior> {
        // TODO: implement me!
        let mut binding = Command::new(target);
        let child = binding.args(args);
        
        unsafe { child.pre_exec(|| child_traceme()) };
        let child_process = child.spawn().ok()?;
        let pid = Pid::from_raw(child_process.id() as i32);
        waitpid(pid, None).ok()?;
        let mut inferior = Inferior { child: child_process, breakpoints: HashMap::new() };
        for breakpoint in breakpoints {
            let orig_byte = inferior.write_byte(*breakpoint, 0xcc).ok().unwrap();
            inferior.breakpoints.insert(*breakpoint, Breakpoint { addr: *breakpoint, orig_byte });
        }
        Some(inferior)
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    /// Resumes the execution of a traced process.
    ///
    /// This function sends a request to continue the execution of a traced child process,
    /// and then waits for the process to generate a result. It combines the continuation
    /// and waiting operations into an atomic operation.
    ///
    /// # Returns
    /// - `Result<Status, nix::Error>`: Returns a result type containing either the status information
    ///   of the child process or an error encountered during the operation.
    ///
    /// # Error Handling
    /// - `ptrace::cont(self.pid(), None)?`: If the continue operation fails, it returns an error.
    /// - `self.wait(None)`: The wait operation may have different outcomes based on the state of the child process.
    pub fn cont(&mut self) -> Result<Status, nix::Error> {
        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    pub fn print_backtrace(&mut self, dwarf_data: &DwarfData) -> Result<(), nix::Error> {
        let regs = ptrace::getregs(self.pid()).unwrap();
        let mut instruction_ptr = regs.rip as usize;
        let mut base_ptr = regs.rbp as usize;
        loop {
            let func = dwarf_data.get_function_from_addr(instruction_ptr).expect("Function not found");
            let line_num = dwarf_data.get_line_from_addr(instruction_ptr).expect("Line number not found");
            println!("{} ({})", func, line_num);
            if func == "main" {
                break;
            }
            instruction_ptr = ptrace::read(self.pid(), (base_ptr + 8) as ptrace::AddressType)? as usize;
            base_ptr = ptrace::read(self.pid(), base_ptr as ptrace::AddressType)? as usize;
        }
        Ok(())
    }

    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }

    pub fn continue_breakpoint(&mut self, breakpoint: &usize) -> Result<(), nix::Error> {
        ptrace::step(self.pid(), None).ok();
        let pid = self.pid();
        match waitpid(pid, None) {
            Ok(WaitStatus::Stopped(_, nix::sys::signal::Signal::SIGTRAP)) => {
                // Check if the process stopped correctly
                println!("Process stopped at the next instruction");
    
                // Restore the original instruction byte at the breakpoint location
                self.write_byte(*breakpoint, 0xcc).expect("Failed to restore the original instruction byte");
            }
            Ok(WaitStatus::Exited(_, status)) => {
                println!("Process exited with status: {}", status);
                return Ok(());
            }
            Ok(WaitStatus::Signaled(_, signal, _)) => {
                println!("Process was killed by signal: {}", signal);
                return Ok(());
            }
            _ => {
                println!("Unexpected status");
                return Err(nix::Error::from(nix::errno::Errno::EINVAL));
            }
        }
        self.cont().expect("Failed to continue");
        let wait_status = waitpid(self.pid(), None);
            if let Ok(WaitStatus::Stopped(_, nix::sys::signal::Signal::SIGTRAP)) = wait_status {
                let rip = ptrace::getregs(self.pid()).unwrap().rip as usize;
                if rip - 1 == *breakpoint {
                    let orig_byte = self.breakpoints.get(breakpoint).unwrap().orig_byte;
                    self.write_byte(*breakpoint, orig_byte).expect("Failed to restore the original instruction byte");
                    let mut regs = ptrace::getregs(self.pid()).unwrap();
                    regs.rip = (rip - 1) as u64;
                    ptrace::setregs(self.pid(), regs).ok();
                }
            }
        Ok(())

    }

}
