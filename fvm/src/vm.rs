//! Module that contains the VM that executes bytecode

use bigint::{Address, H256, M256, MI256, U256, U128};
use tiny_keccak::Keccak;

use errors::{Result, VMError};
use eth_log::Log;
use libvm::Cpu;
use memory::{Memory, SimpleMemory};
pub use opcodes::Opcode;
use std::array::FixedSizeArray;
use storage::Storage;
use std::collections::HashMap;
use ethereum_types::H160;
use account::Account;
use transaction::Transaction;
use rlp::Encodable;

/// Core VM struct that executes bytecode
pub struct VM {
    accounts: HashMap<H160, Account>,
    account_gas: HashMap<H160, U256>,
    account_code: HashMap<H160, Vec<u8>>,
    address: Option<Address>,
    registers: [M256; 1024],
    memory: Option<Box<dyn Memory>>,
    storage: Option<Storage>,
    code: Vec<u8>,
    pc: usize,
    stack_pointer: usize,
    logs: Vec<Log>,
    current_transaction: Option<Transaction>,
    current_sender: Option<H160>,
}

impl VM {
    /// Creates and returns a new VM
    pub fn new(code: Vec<u8>) -> VM {
        VM {
            accounts: HashMap::new(),
            account_code: HashMap::new(),
            account_gas: HashMap::new(),
            address: None,
            current_transaction: None,
            current_sender: None,
            registers: [0.into(); 1024],
            memory: None,
            storage: None,
            stack_pointer: 0,
            code,
            pc: 0,
            logs: vec![],
        }
    }

    /// Sets the volatile memory of the VM to the SimpleMemory type
    pub fn with_simple_memory(mut self) -> VM {
        self.memory = Some(Box::new(SimpleMemory::new()));
        self
    }

    /// Sets the storage of the VM. This is unique to the Address.
    pub fn with_storage(mut self, address: Address) -> VM {
        self.storage = Some(Storage::new(address));
        self
    }

    /// Sets the address for this VM
    pub fn with_address(mut self, address: Address) -> VM {
        self.address = Some(address);
        self
    }

    /// Creates a VM with a random address, mainly for testing purposes
    pub fn with_random_address(mut self) -> VM {
        self.address = Some(Address::random());
        self
    }

    /// Starts the execution loop for the VM
    pub fn execute(&mut self) -> Result<()> {
        loop {
            match self.execute_one() {
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
    }

    /// Executes the next instruction only
    pub fn execute_one(&mut self) -> Result<()> {
        let opcode = Opcode::from(&self.code[self.pc]);
        self.execute_one_instruction(opcode)
    }

    fn execute_one_instruction(&mut self, opcode: Opcode) -> Result<()> {
        match opcode {
            Opcode::STOP => {
                return Ok(());
            }
            Opcode::ADD => {
                self.stack_pointer -= 1;
                let result = self.registers[self.stack_pointer] + self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = result;
                self.pc += 1;
            }
            Opcode::MUL => {
                self.stack_pointer -= 1;
                let result = self.registers[self.stack_pointer] * self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = result;
                self.pc += 1;
            }
            Opcode::SUB => {
                self.stack_pointer -= 1;
                let result = self.registers[self.stack_pointer] - self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = result;
                self.pc += 1;
            }
            Opcode::DIV => {
                self.stack_pointer -= 1;
                let result = self.registers[self.stack_pointer] / self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = result;
                self.pc += 1;
            }
            Opcode::SDIV => {
                self.stack_pointer -= 1;
                let s1 = MI256::from(self.registers[self.stack_pointer]);
                let s2 = MI256::from(self.registers[self.stack_pointer - 1]);
                let result = s1 / s2;
                let result: M256 = result.into();
                self.registers[self.stack_pointer - 1] = result;
                self.pc += 1;
            }
            Opcode::SMOD => {
                self.stack_pointer -= 1;
                let s1 = MI256::from(self.registers[self.stack_pointer]);
                let s2 = MI256::from(self.registers[self.stack_pointer - 1]);
                let result = s1 / s2;
                self.registers[self.stack_pointer - 1] = result.into();
                self.pc += 1;
            }
            Opcode::MOD => {
                self.stack_pointer -= 1;
                let result = self.registers[self.stack_pointer] % self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = result;
                self.pc += 1;
            }
            Opcode::ADDMOD => {
                self.stack_pointer -= 1;
                let result = (self.registers[self.stack_pointer] + self.registers[self.stack_pointer - 1])
                    % self.registers[self.stack_pointer - 2];
                if result == self.registers[self.stack_pointer - 2] {
                    self.registers[self.stack_pointer - 2] = result;
                } else {
                    self.registers[self.stack_pointer - 2] = 0.into();
                }
            }
            Opcode::MULMOD => {
                self.stack_pointer -= 1;
                let result = (self.registers[self.stack_pointer] * self.registers[self.stack_pointer - 1])
                    % self.registers[self.stack_pointer - 2];
                if result == self.registers[self.stack_pointer - 2] {
                    self.registers[self.stack_pointer - 2] = result;
                } else {
                    self.registers[self.stack_pointer - 2] = 0.into();
                }
            }
            Opcode::EXP => {
                let s1 = self.registers[self.stack_pointer];
                let s2 = self.registers[self.stack_pointer - 1];
                if s1 > M256::from(32) {
                    self.registers[self.stack_pointer - 1] = s2;
                } else {
                    let mut ret = M256::zero();
                    let len: usize = s1.as_usize();
                    let t: usize = 8 * (len + 1) - 1;
                    let t_bit_mask = M256::one() << t;
                    let t_value = (s2 & t_bit_mask) >> t;
                    for i in 0..256 {
                        let bit_mask = M256::one() << i;
                        let i_value = (s2 & bit_mask) >> i;
                        if i <= t {
                            ret = ret + (i_value << i);
                        } else {
                            ret = ret + (t_value << i);
                        }
                    }
                    self.registers[self.stack_pointer - 1] = s2;
                }
            }
            Opcode::SIGNEXTEND => {
                let s1: U256 = self.registers[self.stack_pointer].into();
                if s1 < U256::from(32) {
                    let s2: U256 = self.registers[self.stack_pointer - 1].into();
                    let bit_position = (s2.low_u64() * 8 + 7) as usize;

                    let bit = s2.bit(bit_position);
                    let mask = (U256::one() << bit_position) - U256::one();
                    if bit {
                        self.registers[self.stack_pointer - 1] = (s2 | !mask).into()
                    } else {
                        self.registers[self.stack_pointer - 1] = (s2 & mask).into()
                    };
                }
            }
            Opcode::LT => {
                self.stack_pointer -= 1;
                if self.registers[self.stack_pointer] > self.registers[self.stack_pointer - 1] {
                    self.registers[self.stack_pointer - 1] = 1.into();
                } else {
                    self.registers[self.stack_pointer - 1] = 0.into();
                }
                self.pc += 2;
            }
            Opcode::GT => {
                self.stack_pointer -= 1;
                if self.registers[self.stack_pointer] < self.registers[self.stack_pointer - 1] {
                    self.registers[self.stack_pointer - 1] = 1.into();
                } else {
                    self.registers[self.stack_pointer - 1] = 0.into();
                }
                self.pc += 1;
            }
            Opcode::SLT => {
                self.stack_pointer -= 1;
                let s1 = MI256::from(self.registers[self.stack_pointer]);
                let s2 = MI256::from(self.registers[self.stack_pointer - 1]);
                let result = s1 > s2;
                self.registers[self.stack_pointer - 1] = result.into();
                self.pc += 1;
            }
            Opcode::SGT => {
                self.stack_pointer -= 1;
                let s1 = MI256::from(self.registers[self.stack_pointer]);
                let s2 = MI256::from(self.registers[self.stack_pointer - 1]);
                let result = s1 < s2;
                self.registers[self.stack_pointer - 1] = result.into();
                self.pc += 1;
            }
            Opcode::EQ => {
                self.stack_pointer -= 1;
                if self.registers[self.stack_pointer] == self.registers[self.stack_pointer - 1] {
                    self.registers[self.stack_pointer - 1] = 1.into();
                } else {
                    self.registers[self.stack_pointer - 1] = 0.into();
                }
                self.pc += 1;
            }
            Opcode::ISZERO => {
                self.stack_pointer -= 1;
                if self.registers[self.stack_pointer] == 0.into() {
                    self.registers[self.stack_pointer] = 1.into()
                } else {
                    self.registers[self.stack_pointer] = 0.into()
                }
                self.pc += 1;
            }
            Opcode::AND => {
                self.stack_pointer -= 1;
                let s1 = self.registers[self.stack_pointer];
                let s2 = self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = s1 & s2;
                self.pc += 1;
            }
            Opcode::OR => {
                self.stack_pointer -= 1;
                let s1 = self.registers[self.stack_pointer];
                let s2 = self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = s1 | s2;
                self.pc += 1;
            }
            Opcode::XOR => {
                self.stack_pointer -= 1;
                let s1 = self.registers[self.stack_pointer];
                let s2 = self.registers[self.stack_pointer - 1];
                self.registers[self.stack_pointer - 1] = s1 ^ s2;
                self.pc += 1;
            }
            Opcode::NOT => {
                self.stack_pointer -= 1;
                let s1 = self.registers[self.stack_pointer];
                self.registers[self.stack_pointer] = !s1;
                self.pc += 1;
            }
            Opcode::BYTE => {
                self.stack_pointer -= 1;
                let s1 = self.registers[self.stack_pointer];
                let s2 = self.registers[self.stack_pointer - 1];
                let mut ret = M256::zero();
                for i in 0..256 {
                    if i < 8 && s1 < 32.into() {
                        let o: usize = s1.as_usize();
                        let t = 255 - (7 - i + 8 * o);
                        let bit_mask = M256::one() << t;
                        let value = (s2 & bit_mask) >> t;
                        ret = ret + (value << i);
                    }
                }
                self.registers[self.stack_pointer] = ret;
            }
            Opcode::SHA3 => {
                let offset = self.registers[self.stack_pointer];
                let size = self.registers[self.stack_pointer - 1];
                if let Some(ref mut mem) = self.memory {
                    let mut sha3 = Keccak::new_sha3_256();
                    sha3.update(mem.read_slice(offset.into(), size.into()));
                    let mut k: [u8; 32] = [0; 32];
                    sha3.finalize(&mut k);
                    println!("k is: {:?}", k);
                    self.registers[self.stack_pointer - 1] = M256::from(k.as_ref());
                    self.pc += 1;
                }
            }
            Opcode::ADDRESS => {
                if self.address.is_some() {
                    self.registers[self.stack_pointer] = self.address.unwrap().clone().into();
                }
            }
            Opcode::BALANCE => unimplemented!(),
            Opcode::ORIGIN => unimplemented!(),
            Opcode::CALLER => unimplemented!(),
            Opcode::CALLVALUE => unimplemented!(),
            Opcode::CALLDATALOAD => unimplemented!(),
            Opcode::CALLDATASIZE => unimplemented!(),
            Opcode::CALLDATACOPY => unimplemented!(),
            Opcode::CODESIZE => {
                self.registers[self.stack_pointer] = self.code.len().into();
            }
            Opcode::CODECOPY => {
                let memory_offset: U256 = self.registers[self.stack_pointer].into();
                let code_offset = self.registers[self.stack_pointer - 1];
                let size = self.registers[self.stack_pointer - 2];

                for (i, b) in self
                    .code
                    .iter()
                    .skip(code_offset.as_usize())
                    .take(size.as_usize())
                    .cloned()
                    .enumerate()
                {
                    if let Some(ref mut s) = &mut self.storage {
                        s.write(memory_offset + i.into(), ([b].as_slice()).into())?;
                    } else {
                        return Err(VMError::MemoryError.into());
                    }
                }
            }
            Opcode::GASPRICE => unimplemented!(),
            Opcode::EXTCODESIZE => unimplemented!(),
            Opcode::EXTCODECOPY => unimplemented!(),
            Opcode::RETURNDATACOPY => unimplemented!(),
            Opcode::RETURNDATASIZE => {
                let memory_offset = self.registers[self.stack_pointer];
                let output_offset = self.registers[self.stack_pointer - 1].as_usize();
                let size = self.registers[self.stack_pointer - 2].as_usize();
                if let Some(ref mut mem) = &mut self.memory {
                    for i in 0..size {
                        let value = self.registers[output_offset - i];
                        mem.write(memory_offset + i.into(), value)?;
                    }
                } else {
                    return Err(VMError::MemoryError.into());
                }
            },
            Opcode::PC => {
                self.registers[self.stack_pointer] = (self.pc - 1).into();
            }
            Opcode::POP => {
                self.stack_pointer -= 1;
            }
            Opcode::GAS => {
                self.registers[self.stack_pointer] = self.account_gas.values().fold(M256::from(0), |acc, a| {
                    acc + (*a).into()
                });
            },
            Opcode::JUMP => {
                let new_pc = self.registers[self.stack_pointer];
                self.pc = new_pc.as_usize();
            }
            Opcode::JUMPI => {
                self.stack_pointer -= 1;
                let destination = self.registers[self.stack_pointer];
                let check = self.registers[self.stack_pointer - 1];
                if check.as_usize() == 0 {
                    self.pc = destination.as_usize();
                }
            }
            Opcode::JUMPDEST => {}
            Opcode::CREATE => {
                let bytes = self.registers[self.stack_pointer].rlp_bytes().into_vec();
                let mut id_bytes = [0u8; 20];
                for (n, byte) in bytes.into_iter().take(20).enumerate() {
                    id_bytes[n] = byte;
                }
                let id: H160 = id_bytes.into();
                let start_offset = self.registers[self.stack_pointer-1].into();
                let size = self.registers[self.stack_pointer-2].into();
                if let Some(ref mut store) = self.storage {
                    let mut code = vec![];
                    let mut counter = start_offset;
                    while counter < start_offset + size {
                        code.push(store.read(counter)?.as_u32() as u8);
                        counter = counter + 1.into();
                    }
                    let account = Account::new(format!("{}", id), 0, "".parse().unwrap())?;
                    self.accounts.insert(id.clone(), account);
                    self.account_code.insert(id, code);
                } else {
                    return Err(VMError::MemoryError.into());
                }
            },
            Opcode::CALL => self.execute_call()?,
            Opcode::CALLCODE => {
                let to = self.current_transaction.as_ref().map(|t| t.to.unwrap()).unwrap();
                self.current_sender = Some(to);
                self.execute_call()?
            },
            Opcode::RETURN => {
                self.pc = self.code.len();
                let offset = self.registers[self.stack_pointer];
                let size = self.registers[self.stack_pointer-1];
                if let Some(ref mem) = self.memory {
                    let info = mem.read_slice(offset.into(), size.into());
                    self.registers[self.stack_pointer] = info.into();
                } else {
                    return Err(VMError::MemoryError.into());
                }
            },
            Opcode::DELEGATECALL => self.execute_call()?,
            Opcode::INVALID => {
                Err(VMError::InvalidInstruction)?;
            },
            Opcode::SUICIDE => {
                let from = self.current_sender.unwrap();
                self.pc = self.code.len();
                self.account_code.remove(&from);
                self.accounts.remove(&from);
            },
            Opcode::SLOAD => {
                self.stack_pointer -= 1;
                let s1 = self.registers[self.stack_pointer];
                if let Some(ref mut store) = self.storage {
                    self.registers[self.stack_pointer] = store.read(s1.into()).unwrap();
                } else {
                    return Err(VMError::MemoryError.into());
                }
            }
            Opcode::SSTORE => {
                self.stack_pointer -= 1;
                let s1 = self.registers[self.stack_pointer];
                let s2 = self.registers[self.stack_pointer - 1];
                if let Some(ref mut store) = self.storage {
                    match store.write(s1.into(), s2) {
                        Ok(_) => {}
                        Err(_e) => return Err(VMError::MemoryError.into()),
                    }
                } else {
                    return Err(VMError::MemoryError.into());
                }
            }
            Opcode::MLOAD => {
                self.stack_pointer -= 1;
                let offset = self.registers[self.stack_pointer];
                if let Some(ref mut mem) = self.memory {
                    self.registers[self.stack_pointer] = mem.read(offset);
                } else {
                    return Err(VMError::MemoryError.into());
                }
            }
            Opcode::MSTORE => {
                self.stack_pointer -= 1;
                let offset = self.registers[self.stack_pointer];
                let value = self.registers[self.stack_pointer - 1];
                if let Some(ref mut mem) = self.memory {
                    mem.write(offset, value)?;
                    self.pc += 1;
                } else {
                    return Err(VMError::MemoryError.into());
                }
            }
            Opcode::MSTORE8 => {
                self.stack_pointer -= 1;
                let offset = self.registers[self.stack_pointer];
                let value = self.registers[self.stack_pointer - 1] % 256.into();
                if let Some(ref mut mem) = self.memory {
                    mem.write_byte(offset, (value.0.low_u32() & 0xFF) as u8)?;
                    self.pc += 1;
                }
            }
            Opcode::MSIZE => {
                if let Some(ref mut mem) = self.memory {
                    self.registers[self.stack_pointer] = mem.size();
                    self.pc += 1;
                } else {
                    return Err(VMError::MemoryError.into());
                }
            }
            Opcode::PUSH(bytes) => {
                let range = &self.code[self.pc + 1..self.pc + 1 + bytes as usize];
                self.registers[self.stack_pointer] = M256::from(range);
                self.stack_pointer += 1;
                self.pc += bytes as usize + 1;
            }
            Opcode::DUP(bytes) => {
                let val = self.registers[bytes as usize - 1];
                self.registers[self.stack_pointer] = val;
            }
            Opcode::SWAP(bytes) => {
                let val1 = self.registers[self.stack_pointer - 1];
                let val2 = self.registers[bytes as usize - 1];
                self.registers[self.stack_pointer - 1] = val2;
                self.registers[bytes as usize - 1] = val1;
            }
            Opcode::LOG(bytes) => {
                self.stack_pointer -= 1;
                let index = self.registers[self.stack_pointer];
                let len = self.registers[self.stack_pointer - 1];
                if let Some(ref mut mem) = self.memory {
                    let data = mem.copy_from_memory(index.into(), len.into());
                    let mut topics: Vec<H256> = Vec::new();
                    for _ in 0..bytes {
                        let pointer = self.stack_pointer + (bytes as usize + 1);
                        topics.push(H256::from(self.registers[pointer]));
                    }
                    println!("Pushing logs");
                    self.logs.push(Log {
                        address: self.address.unwrap(),
                        data,
                        topics,
                    });
                } else {
                    return Err(VMError::MemoryError.into());
                }
            }
            _ => unimplemented!(),
        };
        Ok(())
    }

    fn execute_call(&mut self) -> Result<()> {
        let from = self.current_sender.unwrap();
        let to_bytes = self.registers[self.stack_pointer].rlp_bytes().into_vec();
        let mut id_bytes = [0u8; 20];
        for (n, byte) in to_bytes.into_iter().take(20).enumerate() {
            id_bytes[n] = byte;
        }
        let to: H160 = id_bytes.into();
        let new_code = self.account_code[&to].clone();
        let old_code = self.code.clone();
        let old_pc = self.pc;
        self.code = new_code;
        self.pc = 0;
        self.execute()?;
        self.code = old_code;
        self.pc = old_pc;
        let in_offset = self.registers[self.stack_pointer - 3];
        let in_size = self.registers[self.stack_pointer - 4];
        let out_offset = self.registers[self.stack_pointer - 5];
        let out_size = self.registers[self.stack_pointer - 6];
        if let Some(ref mut mem) = self.memory {
            let slice = mem.read_slice(out_offset.into(), in_size.into());
            self.registers[self.stack_pointer - 6] = slice.into();
            Ok(())
        } else {
            return Err(VMError::MemoryError.into());
        }
    }

    /// Utility function to print the values of a range of registers
    pub fn print_registers(&self, start: usize, end: usize) {
        println!("Stack Pointer is: {:?}", self.stack_pointer);
        println!("Registers are: ");
        for register in self.registers[start..end].iter() {
            print!("{:?} ", register);
        }
        println!("\nEnd of Registers");
    }
}

impl Default for VM {
    fn default() -> VM {
        VM {
            // In stack-based EVM implementations, the stack has a limit of 1024 items. This is why
            // there is a limit of 1024 registers.
            registers: [0.into(); 1024],
            memory: Some(Box::new(SimpleMemory::new())),
            storage: None,
            stack_pointer: 0,
            code: vec![],
            pc: 0,
            logs: vec![],
            accounts: HashMap::default(),
            account_code: HashMap::default(),
            account_gas: HashMap::default(),
            current_transaction: None,
            current_sender: None,
            address: None,
        }
    }
}

impl Cpu<Opcode, H160> for VM {
    fn execute_instruction(&mut self, instruction: Opcode) -> Result<()> {
        self.execute_one_instruction(instruction)
    }

    fn get_pc(&self) -> usize {
        self.pc
    }

    fn get_next_instruction(&mut self) -> Option<Opcode> {
        if self.is_done() {
            Some(Opcode::from(&self.code[self.pc]))
        } else {
            None
        }
    }

    fn can_run(&self) -> bool {
        true
    }

    fn is_done(&self) -> bool {
        self.pc < self.code.len()
    }

    fn increase_pc(&mut self, steps: usize) {
        self.pc += steps;
    }

    fn set_instructions<J: Iterator<Item = Opcode>>(&mut self, i: J, sender: H160) {
        let bytes: Vec<u8> = i.map(Opcode::into).collect();
        let transaction: Transaction = serde_json::from_slice(&bytes).unwrap();
        let code = transaction.data.clone();
        self.code = code;
        self.current_transaction = Some(transaction);
        self.current_sender = Some(sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vm() {
        let default_code = vec![0];
        let vm = VM::new(default_code);
        assert_eq!(vm.registers.len(), 1024);
    }

    #[test]
    fn test_stop_opcode() {
        let default_code = vec![0];
        let mut vm = VM::new(default_code);
        assert!(vm.execute_one().is_ok())
    }

    #[test]
    fn test_push_opcode() {
        let default_code = vec![0x60, 0xa];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 10.into());
    }

    #[test]
    fn test_add_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xa, 0x01];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 20.into());
    }

    #[test]
    fn test_sub_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xa, 0x03];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 0.into());
    }

    #[test]
    fn test_mul_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xa, 0x02];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 100.into());
    }

    #[test]
    fn test_div_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xa, 0x04];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 1.into());
    }

    #[test]
    fn test_sdiv_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xa, 0x05];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        vm.print_registers(0, 10);
        assert_eq!(vm.registers[0], 1.into());
    }

    #[test]
    fn test_smod_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xb, 0x07];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 1.into());
    }

    #[test]
    fn test_mod_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xb, 0x06];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 1.into());
    }

    #[test]
    fn test_lt_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xb, 0x10];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 1.into());
    }

    #[test]
    fn test_gt_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xb, 0x11];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 0.into());
    }

    #[test]
    fn test_bitwise_and_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xb, 0x16];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 10.into());
    }

    #[test]
    fn test_bitwise_or_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xb, 0x17];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 11.into());
    }

    #[test]
    fn test_bitwise_xor_opcode() {
        let default_code = vec![0x60, 0xa, 0x60, 0xb, 0x18];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 1.into());
    }

    #[test]
    fn test_addmod_opcode() {
        let default_code = vec![0x60, 0x0d, 0x60, 0x03, 0x60, 0x05, 0x08];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 13.into());
    }

    #[test]
    fn test_mulmod_opcode() {
        let default_code = vec![0x60, 0x10, 0x60, 0x05, 0x60, 0x05, 0x09];
        let mut vm = VM::new(default_code);
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], 16.into());
    }

    #[test]
    fn test_memstore_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x01, 0x52];
        let mut vm = VM::new(default_code).with_simple_memory();
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let memory = vm.memory.unwrap();
        assert!(memory.size() > 0.into());
    }

    #[test]
    fn test_memstore8_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x01, 0x53];
        let mut vm = VM::new(default_code).with_simple_memory();
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let memory = vm.memory.unwrap();
        assert!(memory.size() > 0.into());
    }

    #[test]
    fn test_memload_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x01, 0x52, 0x01, 0x51];
        let mut vm = VM::new(default_code).with_simple_memory();
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], M256::from(5));
    }

    #[test]
    fn test_dup_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x01, 0x80];
        let mut vm = VM::new(default_code).with_simple_memory();
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[2], M256::from(5));
    }

    #[test]
    fn test_swap_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x01, 0x90];
        let mut vm = VM::new(default_code).with_simple_memory();
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert_eq!(vm.registers[0], M256::from(1));
        assert_eq!(vm.registers[1], M256::from(5));
    }

    #[test]
    fn test_log_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x01, 0x60, 0x00, 0x60, 0x01, 0xa1];
        let mut vm = VM::new(default_code).with_simple_memory().with_random_address();
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        let result = vm.execute_one();
        assert!(result.is_ok());
        assert!(vm.logs.len() > 0);
    }

    #[test]
    fn test_sload_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x01, 0x54];
        let mut vm = VM::new(default_code).with_simple_memory().with_random_address();
        vm.storage = Some(Storage::new(vm.address.unwrap()));
        if let Some(ref mut store) = vm.storage {
            assert!(store.write(0.into(), 100.into()).is_ok());
        };
        assert!(vm.execute_one().is_ok());
        assert!(vm.execute_one().is_ok());
    }

    #[test]
    fn test_store_opcode() {
        let default_code = vec![0x60, 0x00, 0x60, 0x05, 0x55];
        let mut vm = VM::new(default_code).with_simple_memory().with_random_address();
        vm.storage = Some(Storage::new(vm.address.unwrap()));
        assert!(vm.execute_one().is_ok());
        assert!(vm.execute_one().is_ok());
    }

    #[test]
    fn test_sha3_opcode() {
        let default_code = vec![0x60, 0x05, 0x60, 0x00, 0x52, 0x20];
        let mut vm = VM::new(default_code).with_simple_memory().with_random_address();
        assert!(vm.execute_one().is_ok());
        assert!(vm.execute_one().is_ok());
        assert!(vm.execute_one().is_ok());
        assert!(vm.execute_one().is_ok());
    }
}
