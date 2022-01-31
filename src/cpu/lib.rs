use core::ops::{Add, Mul};

use std::fmt::Display;

use std::num::TryFromIntError;

pub const NUM_PARAMS: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct Word(pub i64);

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<i64> for Word {
    type Output = Word;

    fn add(self, rhs: i64) -> Self::Output {
        Word(self.0 + rhs)
    }
}

impl Add<usize> for Word {
    type Output = Word;

    fn add(self, rhs: usize) -> Self::Output {
        match rhs.try_into() {
            Ok(n) => {
                if let Some(total) = self.0.checked_add(n) {
                    Word(total)
                } else {
                    panic!("arithmetic overflow in word addition");
                }
            }
            Err(_) => {
                panic!(
                    "cannot add {} to {} as {} does not fit into i64",
                    rhs, self.0, rhs
                );
            }
        }
    }
}

impl Add for Word {
    type Output = Word;

    fn add(self, rhs: Self) -> Self::Output {
        self + rhs.0
    }
}

impl Mul for Word {
    type Output = Word;

    fn mul(self, rhs: Self) -> Self::Output {
        Word(self.0 * rhs.0)
    }
}

impl TryFrom<Word> for usize {
    type Error = TryFromIntError;
    fn try_from(w: Word) -> Result<Self, Self::Error> {
        usize::try_from(w.0)
    }
}

impl From<Word> for bool {
    fn from(w: Word) -> Self {
        w.0 != 0
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Word {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

pub enum AddressingMode {
    POSITIONAL,
    IMMEDIATE,
    RELATIVE,
}

enum Opcode {
    Add = 1,       // day 2
    Multiply = 2,  // day 2
    Read = 3,      // day 5,
    Write = 4,     // day 5
    JumpTrue = 5,  // day 5 part 2
    JumpFalse = 6, // day 5 part 2
    CmpLess = 7,   // day 5 part 2
    CmpEq = 8,     // day 5 part 2
    DeltaRelBase = 9,
    Stop = 99, // day 2
}

#[derive(Debug)]
pub struct BadOpcode {
    code: i64,
}

impl Display for BadOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad opcode {}", self.code)
    }
}

impl std::error::Error for BadOpcode {}

impl TryFrom<&Word> for Opcode {
    type Error = BadOpcode;

    fn try_from(instruction: &Word) -> Result<Opcode, BadOpcode> {
        let opcode = instruction.0 % 100;
        match opcode {
            1 => Ok(Opcode::Add),
            2 => Ok(Opcode::Multiply),
            3 => Ok(Opcode::Read),
            4 => Ok(Opcode::Write),
            5 => Ok(Opcode::JumpTrue),
            6 => Ok(Opcode::JumpFalse),
            7 => Ok(Opcode::CmpLess),
            8 => Ok(Opcode::CmpEq),
            9 => Ok(Opcode::DeltaRelBase),
            99 => Ok(Opcode::Stop),
            _ => Err(BadOpcode { code: opcode }),
        }
    }
}

struct DecodedInstruction {
    op: Opcode,
    addressing_modes: [AddressingMode; NUM_PARAMS],
}

#[derive(Debug)]
pub struct BadAddressingMode {
    mode: i64,
}

impl Display for BadAddressingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad parameter mode {}", self.mode)
    }
}

impl std::error::Error for BadAddressingMode {}

#[derive(Debug)]
pub enum BadInstructionKind {
    BadOp(BadOpcode),
    BadAddrMode(BadAddressingMode),
}

impl Display for BadInstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BadInstructionKind::BadOp(opcode) => write!(f, "{}", opcode),
            BadInstructionKind::BadAddrMode(mode) => write!(f, "{}", mode),
        }
    }
}

#[derive(Debug)]
pub struct BadInstruction {
    kind: BadInstructionKind,
    instruction: Word,
}

impl Display for BadInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad instruction {}: {}", &self.instruction, &self.kind)
    }
}

impl std::error::Error for BadInstruction {}

impl TryFrom<&i64> for AddressingMode {
    type Error = BadAddressingMode;

    fn try_from(instruction: &i64) -> Result<Self, Self::Error> {
        let mode = instruction % 10;
        match mode {
            0 => Ok(AddressingMode::POSITIONAL),
            1 => Ok(AddressingMode::IMMEDIATE),
            2 => Ok(AddressingMode::RELATIVE),
            _ => Err(BadAddressingMode { mode }),
        }
    }
}

fn getmodes(m: &i64) -> Result<[AddressingMode; NUM_PARAMS], BadAddressingMode> {
    // The units and tens digits of the instruction are the opcode.
    // The 3 modes are (index 1) the hundreds, (index 2) thousands and
    // (index 3) the ten-thousands digit.
    let m1: AddressingMode = (&(m / 100)).try_into()?;
    let m2: AddressingMode = (&(m / 1000)).try_into()?;
    let m3: AddressingMode = (&(m / 10000)).try_into()?;
    Ok([
        AddressingMode::POSITIONAL, // never used
        m1,
        m2,
        m3,
    ])
}

impl TryFrom<&Word> for DecodedInstruction {
    type Error = BadInstruction;

    fn try_from(instruction: &Word) -> Result<Self, Self::Error> {
        let op: Opcode = instruction.try_into().map_err(|e| BadInstruction {
            instruction: *instruction,
            kind: BadInstructionKind::BadOp(e),
        })?;
        let addressing_modes = getmodes(&instruction.0).map_err(|e| BadInstruction {
            instruction: *instruction,
            kind: BadInstructionKind::BadAddrMode(e),
        })?;
        Ok(DecodedInstruction {
            op,
            addressing_modes,
        })
    }
}

fn decode(insruction: Word) -> Result<DecodedInstruction, BadInstruction> {
    (&insruction).try_into()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CpuStatus {
    Halt,
    Run,
}

pub struct Memory {
    content: Vec<Word>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory::new()
    }
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            content: Vec::new(),
        }
    }

    fn pos(addr: Word) -> usize {
        match addr.0.try_into() {
            Ok(a) => a,
            Err(e) => {
                panic!("invalid address {}: {}", addr.0, e);
            }
        }
    }

    pub fn fetch(&self, addr: Word) -> Word {
        let addr: usize = Memory::pos(addr);
        *self.content.get(addr).unwrap_or(&Word(0))
    }

    pub fn store(&mut self, addr: Word, value: Word) {
        self.extend_to_include(addr);
        let addr: usize = Memory::pos(addr);
        self.content[addr] = value;
    }

    pub fn load(&mut self, base: Word, program: &[Word]) {
        self.content.clear();
        self.extend_to_include(base);
        self.content.extend(program);
    }

    pub fn dump(&self, dest: &mut Vec<Word>) {
        dest.clear();
        dest.extend(&self.content);
    }

    pub fn extend_to_include(&mut self, addr: Word) {
        match addr.try_into() {
            Ok(n) if self.content.len() < n => {
                self.content.resize(n, Word(0));
            }
            Ok(_) => {
                // Nothing to do.
            }
            Err(e) => {
                panic!("extend_to_include: invalid location {}: {}", addr.0, e);
            }
        }
    }
}

pub struct Processor {
    ram: Memory,
    relative_base: i64,
    pc: Word,
}

impl Processor {
    pub fn new(initial_pc: Word) -> Processor {
        Processor {
            ram: Memory::new(),
            relative_base: 0,
            pc: initial_pc,
        }
    }

    fn update_relative_base(&mut self, delta: Word) {
        if let Some(updated) = self.relative_base.checked_add(delta.0) {
            self.relative_base = updated;
        } else {
            panic!("i64 overflow in update_relative_base");
        }
    }

    fn set_pc(&mut self, addr: Word) {
        self.pc = addr;
    }

    fn execute_arithmetic_instruction<F: Fn(Word, Word) -> Word>(
        &mut self,
        modes: &[AddressingMode; NUM_PARAMS],
        calculate: F,
    ) {
        let result = calculate(self.get(modes, 1), self.get(modes, 2));
        self.put(modes, 3, result);
    }

    pub fn execute_instruction<FI, FO>(
        &mut self,
        get_input: &mut FI,
        do_output: &mut FO,
    ) -> Result<CpuStatus, BadInstruction>
    where
        FI: FnMut() -> Word,
        FO: FnMut(Word),
    {
        let instruction = self.ram.fetch(self.pc);
        let decoded = decode(instruction)?;
        let (state, next_pc) = match decoded.op {
            Opcode::Add => {
                self.execute_arithmetic_instruction(&decoded.addressing_modes, |a, b| a + b);
                (CpuStatus::Run, self.pc + 4_i64)
            }
            Opcode::Multiply => {
                self.execute_arithmetic_instruction(&decoded.addressing_modes, |a, b| a * b);
                (CpuStatus::Run, self.pc + 4_i64)
            }
            Opcode::Read => {
                let input = get_input();
                self.put(&decoded.addressing_modes, 1, input);
                (CpuStatus::Run, self.pc + 2_i64)
            }
            Opcode::Write => {
                do_output(self.get(&decoded.addressing_modes, 1));
                (CpuStatus::Run, self.pc + 2_i64)
            }
            Opcode::JumpTrue => {
                let val: Word = self.get(&decoded.addressing_modes, 1);
                let next_pc = if val.0 != 0 {
                    self.get(&decoded.addressing_modes, 2)
                } else {
                    self.pc + 3_i64
                };
                (CpuStatus::Run, next_pc)
            }
            Opcode::JumpFalse => {
                let val: Word = self.get(&decoded.addressing_modes, 1);
                let next_pc = if val.0 == 0 {
                    self.get(&decoded.addressing_modes, 2)
                } else {
                    self.pc + 3_i64
                };
                (CpuStatus::Run, next_pc)
            }
            Opcode::CmpLess => {
                let less: bool =
                    self.get(&decoded.addressing_modes, 1) < self.get(&decoded.addressing_modes, 2);
                self.put(&decoded.addressing_modes, 3, Word(if less { 1 } else { 0 }));
                (CpuStatus::Run, self.pc + 4_i64)
            }
            Opcode::CmpEq => {
                let equal: bool = self.get(&decoded.addressing_modes, 1)
                    == self.get(&decoded.addressing_modes, 2);
                self.put(
                    &decoded.addressing_modes,
                    3,
                    Word(if equal { 1 } else { 0 }),
                );
                (CpuStatus::Run, self.pc + 4_i64)
            }
            Opcode::DeltaRelBase => {
                let base = self.get(&decoded.addressing_modes, 1);
                self.update_relative_base(base);
                (CpuStatus::Run, self.pc + 2_i64)
            }
            Opcode::Stop => (CpuStatus::Halt, self.pc),
        };
        self.pc = next_pc;
        Ok(state)
    }

    fn get(&mut self, modes: &[AddressingMode; NUM_PARAMS], index: usize) -> Word {
        assert!(matches!(index, 1 | 2 | 3));
        match modes[index] {
            AddressingMode::POSITIONAL => self.ram.fetch(self.ram.fetch(self.pc + index)),
            AddressingMode::IMMEDIATE => self.ram.fetch(self.pc + index),
            AddressingMode::RELATIVE => self
                .ram
                .fetch(self.ram.fetch(self.pc + index) + self.relative_base),
        }
    }

    fn put(&mut self, modes: &[AddressingMode; NUM_PARAMS], index: usize, value: Word) {
        assert!(matches!(index, 1 | 2 | 3));
        let fetch_loc = self.pc + index;
        let store_loc = match modes[index] {
            AddressingMode::POSITIONAL => self.ram.fetch(fetch_loc),
            AddressingMode::RELATIVE => self.ram.fetch(fetch_loc) + self.relative_base,
            _ => panic!("Immediate addressing mode invalid for store operations"),
        };
        self.ram.store(store_loc, value);
    }

    pub fn ram(&self) -> Vec<Word> {
        let mut result = Vec::new();
        self.ram.dump(&mut result);
        result
    }

    pub fn load(&mut self, base: Word, content: &[Word]) {
        self.ram.load(base, content);
    }

    pub fn run_with_io<FI, FO>(
        &mut self,
        get_input: &mut FI,
        do_output: &mut FO,
    ) -> Result<(), BadInstruction>
    where
        FI: FnMut() -> Word,
        FO: FnMut(Word),
    {
        while self.execute_instruction(get_input, do_output)? == CpuStatus::Run {
            // No need to do anything in the body.
        }
        Ok(())
    }

    pub fn run_with_fixed_input<FO>(
        &mut self,
        fixed_input: &[Word],
        mut do_output: FO,
    ) -> Result<(), BadInstruction>
    where
        FO: Fn(Word) + Copy,
    {
        let mut it = fixed_input.iter();
        let mut get_input = || -> Word {
            if let Some(val) = it.next() {
                *val
            } else {
                panic!("run_with_fixed_input: ran out of input")
            }
        };
        loop {
            match self.execute_instruction(&mut get_input, &mut do_output) {
                Ok(CpuStatus::Run) => (),
                Ok(CpuStatus::Halt) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
fn assert_same(label: &str, expected: &[Word], got: &[Word]) {
    if !expected.is_empty() {
        for (i, (e, g)) in expected.iter().zip(got.iter()).enumerate() {
            if e != g {
                panic!(
                    "{} mismatch at location {}: expected {}, got {}",
                    label, i, e.0, g.0
                );
            }
        }
    }
}

#[cfg(test)]
fn check_program(program: &[i64], input: &[i64], expected_ram: &[i64], expected_output: &[i64]) {
    fn w(n: &i64) -> Word {
        Word(*n)
    }
    let w_program: Vec<Word> = program.iter().map(w).collect();
    let w_input: Vec<Word> = input.iter().map(w).collect();
    let w_expected_ram: Vec<Word> = expected_ram.iter().map(w).collect();
    let w_expected_output: Vec<Word> = expected_output.iter().map(w).collect();

    let mut it = w_input.iter();
    let mut get_input = || -> Word {
        if let Some(val) = it.next() {
            *val
        } else {
            panic!("run_with_fixed_input: ran out of input")
        }
    };
    let mut output = Vec::new();
    let mut do_output = |w: Word| {
        output.push(w);
    };

    let mut cpu = Processor::new(Word(0));
    cpu.load(Word(0), &w_program);
    println!("Loaded {}-word program", w_program.len());
    if let Err(e) = cpu.run_with_io(&mut get_input, &mut do_output) {
        panic!("test program contains a bad instruction: {}", e);
    };
    println!("program has completed successfully");
    let ram = cpu.ram();
    for (i, w) in ram.iter().enumerate() {
        println!("ram location {} contains {}", i, w);
    }
    assert_same("ram", &w_expected_ram, &ram);
    assert_same("output", &w_expected_output, &output);
}

#[test]
fn test_cpu() {
    fn check(program: &[i64], expected_memory: &[i64]) {
        check_program(program, &[], expected_memory, &[]);
    }

    check(&[1, 0, 0, 0, 99], &[2, 0, 0, 0, 99]); // from day 2
    check(&[2, 3, 0, 3, 99], &[2, 3, 0, 6, 99]); // from day 2
    check(&[2, 4, 4, 5, 99, 0], &[2, 4, 4, 5, 99, 9801]); // from day 2
    check(
        &[1, 1, 1, 4, 99, 5, 6, 0, 99],
        &[30, 1, 1, 4, 2, 5, 6, 0, 99],
    ); // from day 2
    panic!("just testing");
}
