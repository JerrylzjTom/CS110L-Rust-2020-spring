use crate::debugger_command::DebuggerCommand;
use crate::inferior::{self, Inferior};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::dwarf_data::{DwarfData, Error as DwarfError};

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    dwarf_data: DwarfData,
    breakpoints: Vec<usize>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        // println!("{:?}", debug_data);
        // debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            dwarf_data: debug_data,
            breakpoints: vec![],
        }


    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if self.inferior.is_some() {
                        if let Some(inferior) = self.inferior.as_mut() {
                            // If there is already a running process, kill it
                            inferior.kill().expect("Failed to kill inferior");
                            println!("Killing running inferior (pid {})", inferior.pid());
                            self.inferior = None;
                        }
                    }
                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.breakpoints) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        let inferior = self.inferior.as_mut().unwrap();
                        
                        let status = inferior.cont().ok();
                        if status.is_none() {
                            println!("Error running inferior");
                        }else {
                            match status.unwrap() {
                                inferior::Status::Exited(exit_code) => {
                                    println!("Child exited (status {})", exit_code);
                                    self.inferior = None;
                                }
                                inferior::Status::Signaled(signal) => {
                                    println!("Child signaled (signal {})", signal.to_string());
                                }
                               inferior::Status::Stopped(signal, address) => {
                                    let line_num = self.dwarf_data.get_line_from_addr(address).unwrap();
                                    println!("Child stopped (signal {})", signal.to_string());
                                    println!("Stopped at {}", line_num);
                               }
                            }
                        }
                    }
                    else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    if self.inferior.is_some() {
                        if let Some(inferior) = self.inferior.as_mut() {
                            // If there is already a running process, kill it
                            inferior.kill().expect("Failed to kill inferior");
                            println!("Killing running inferior (pid {})", inferior.pid());
                            self.inferior = None;
                        }
                    }
                    return;
                }
                DebuggerCommand::Continue => {
                    if let Some(inferior) = self.inferior.as_mut() {
                        // TODO (milestone 2): Implement the continue command
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        let status = inferior.cont().ok();
                        if status.is_none() {
                            println!("No process running");
                        }else {
                            match status.unwrap() {
                                inferior::Status::Exited(exit_code) => {
                                    println!("Child exited (status {})", exit_code);
                                }
                                inferior::Status::Signaled(signal) => {
                                    println!("Child signaled (signal {})", signal.to_string());
                                }
                               inferior::Status::Stopped(signal, address) => {
                                    println!("Child stopped (signal {}) at address {}", signal.to_string(), address);
                                    if let Some(line_num) = self.dwarf_data.get_line_from_addr(address) {
                                        println!("Stopped at {}",line_num);
                                    }
                                    if inferior.breakpoints.contains_key(&address) {
                                        inferior.continue_breakpoint(&address).ok();
                                    }
                               }
                            }
                        }
                    }
                }
                DebuggerCommand::Backtrace => {
                    if let Some(inferior) = self.inferior.as_mut() {
                        inferior.print_backtrace(&self.dwarf_data).expect("Could not print backtrace");
                    }
                }
                DebuggerCommand::BreakPoint(args) => {
                        // Check if the string starts with an asterisk
                    if args.starts_with('*') {
                        // Extract the substring after the asterisk
                        let address_str = &args[1..];
                        let address = Self::parse_address(address_str);
                        if address.is_some() {
                            let address = address.unwrap();
                            self.breakpoints.push(address);
                            println!("Setting breakpoint {} at {}", self.breakpoints.len(), address);
                        }
                    } else {
                        if let Some(line_num) = args.parse::<usize>().ok() {
                            // Get the address of the line number
                            let address = self.dwarf_data.get_addr_for_line(None, line_num);
                            if address.is_some() {
                                let address = address.unwrap();
                                self.breakpoints.push(address);
                                println!("Settingbreakpoint {} at {}", self.breakpoints.len(), address);
                            }
                        }else {
                            let address = self.dwarf_data.get_addr_for_function(None, &args);
                            if address.is_some() {
                                let address = address.unwrap();
                                self.breakpoints.push(address);
                                println!("Settingbreakpoint {} at {}", self.breakpoints.len(), address);
                            }else {
                                println!("Could not find address for {}", args);
                            }
                        }
                    }
                }
            }

        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
    /// Parses a hexadecimal address string into its corresponding usize value.
    /// 
    /// # Arguments
    /// - `addr`: A hexadecimal address string, which may or may not have a "0x" prefix.
    /// 
    /// # Returns
    /// - `Option<usize>`: If the parsing is successful, returns a Some wrapping the usize value; otherwise, returns None.
    /// 
    /// # Description
    /// This function first removes any "0x" prefix from the address string, then attempts to parse the remaining string as a usize.
    /// Since the input might be an invalid hexadecimal number, the `ok()` method is used to handle potential errors in the parsing process, returning an Option type to safely manage error cases.
    fn parse_address(addr: &str) -> Option<usize> {
        // Remove any "0x" prefix from the address string
        let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
            &addr[2..]
        } else {
            &addr
        };
        usize::from_str_radix(addr_without_0x, 16).ok()
    }

}
