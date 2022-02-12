use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io;
use std::io::BufRead;
use std::num::{ParseIntError, TryFromIntError};

pub const NUM_PARAMS: usize = 4;

#[derive(Clone, Copy, Hash)]
pub struct Word(pub i64);

impl Word {
    fn checked_add(&self, other: &Word) -> Result<Word, CpuFault> {
        match self.0.checked_add(other.0) {
            Some(total) => Ok(Word(total)),
            None => Err(CpuFault::Overflow),
        }
    }

    fn checked_add_usize(&self, other: &usize) -> Result<Word, CpuFault> {
        let n: i64 = match i64::try_from(*other) {
            Ok(x) => x,
            Err(_) => {
                return Err(CpuFault::Overflow);
            }
        };
        match self.0.checked_add(n) {
            Some(total) => Ok(Word(total)),
            None => Err(CpuFault::Overflow),
        }
    }

    fn checked_mul(&self, other: &Word) -> Result<Word, CpuFault> {
        match self.0.checked_mul(other.0) {
            Some(product) => Ok(Word(product)),
            None => Err(CpuFault::Overflow),
        }
    }
}

fn add(a: Word, b: Word) -> Result<Word, CpuFault> {
    a.checked_add(&b)
}

fn mul(a: Word, b: Word) -> Result<Word, CpuFault> {
    a.checked_mul(&b)
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BadAddressingMode {
    mode: i64,
}

impl Display for BadAddressingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad parameter mode {}", self.mode)
    }
}

impl std::error::Error for BadAddressingMode {}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub struct BadInstruction {
    kind: BadInstructionKind,
    instruction: Word,
    address: Option<Word>,
}

impl Display for BadInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad instruction {}: {}", &self.instruction, &self.kind)
    }
}

impl std::error::Error for BadInstruction {}

#[derive(Clone, Copy, Debug)]
pub enum InputOutputError {
    NoInput,
}

impl Display for InputOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputOutputError::NoInput => f.write_str("ran out of input"),
        }
    }
}

impl std::error::Error for InputOutputError {}

#[derive(Clone, Copy, Debug)]
pub enum CpuFault {
    Overflow,
    InvalidInstruction(BadInstruction),
    MemoryFault,
    AddressingModeNotValidInContext,
    IOError(InputOutputError),
}

impl From<BadInstruction> for CpuFault {
    fn from(bi: BadInstruction) -> Self {
        CpuFault::InvalidInstruction(bi)
    }
}

impl Display for CpuFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuFault::Overflow => f.write_str("arithmetic overflow"),
            CpuFault::InvalidInstruction(bi) => write!(f, "{}", bi),
            CpuFault::MemoryFault => write!(f, "memory fault"),
            CpuFault::AddressingModeNotValidInContext => {
                f.write_str("addressing mode not valid in context")
            }
            CpuFault::IOError(e) => {
                write!(f, "I/O error: {}", e)
            }
        }
    }
}

impl std::error::Error for CpuFault {}

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

impl Eq for Word {}

impl PartialOrd for Word {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Word {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    POSITIONAL,
    IMMEDIATE,
    RELATIVE,
}

#[derive(Debug)]
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug)]
struct DecodedInstruction {
    op: Opcode,
    addressing_modes: [AddressingMode; NUM_PARAMS],
}

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
            kind: BadInstructionKind::BadOp(e),
            instruction: *instruction,
            address: None,
        })?;
        let addressing_modes = getmodes(&instruction.0).map_err(|e| BadInstruction {
            instruction: *instruction,
            kind: BadInstructionKind::BadAddrMode(e),
            address: None,
        })?;
        Ok(DecodedInstruction {
            op,
            addressing_modes,
        })
    }
}

fn decode(insruction: Word, pc: Word) -> Result<DecodedInstruction, BadInstruction> {
    match (&insruction).try_into() {
        Ok(d) => Ok(d),
        Err(mut e) => {
            e.address = Some(pc);
            Err(e)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CpuStatus {
    Halt,
    Run,
}

#[derive(Debug)]
pub struct Memory {
    content: BTreeMap<Word, Word>,
    top: i64,
}

impl Default for Memory {
    fn default() -> Self {
        Memory::new()
    }
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            content: BTreeMap::new(),
            top: 0,
        }
    }

    fn pos(addr: Word) -> Result<Word, CpuFault> {
        if addr.0 < 0 {
            Err(CpuFault::MemoryFault)
        } else {
            Ok(addr)
        }
    }

    pub fn fetch(&self, addr: Word) -> Result<Word, CpuFault> {
        let addr = Memory::pos(addr)?;
        Ok(*self.content.get(&addr).unwrap_or(&Word(0)))
    }

    pub fn store(&mut self, addr: Word, value: Word) -> Result<(), CpuFault> {
        let addr = Memory::pos(addr)?;
        self.content.insert(addr, value);
        self.top = max(self.top, addr.0);
        Ok(())
    }

    pub fn load(&mut self, base: Word, program: &[Word]) -> Result<(), CpuFault> {
        self.content.clear();
        let base: Word = Memory::pos(base)?;
        for (offset, w) in program.iter().enumerate() {
            let offset: Word = match offset.try_into() {
                Ok(n) if n >= 0 => Word(n),
                _ => {
                    return Err(CpuFault::MemoryFault);
                }
            };
            let addr = Word(base.0 + offset.0);
            self.content.insert(addr, *w);
            self.top = max(self.top, addr.0);
        }
        Ok(())
    }

    pub fn dump(&self, dest: &mut Vec<Word>) {
        dest.clear();
        let zero: Word = Word(0);
        if !self.content.is_empty() {
            dest.extend((0..=self.top).map(|addr| self.content.get(&Word(addr)).unwrap_or(&zero)));
        }
    }
}

#[derive(Debug)]
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

    fn update_relative_base(&mut self, delta: Word) -> Result<(), CpuFault> {
        if let Some(updated) = self.relative_base.checked_add(delta.0) {
            self.relative_base = updated;
            Ok(())
        } else {
            Err(CpuFault::Overflow)
        }
    }

    pub fn set_pc(&mut self, addr: Word) {
        self.pc = addr;
    }

    fn execute_arithmetic_instruction<F: Fn(Word, Word) -> Result<Word, CpuFault>>(
        &mut self,
        modes: &[AddressingMode; NUM_PARAMS],
        calculate: F,
    ) -> Result<(), CpuFault> {
        match calculate(self.get(modes, 1)?, self.get(modes, 2)?) {
            Ok(result) => {
                self.put(modes, 3, result)?;
                Ok(())
            }
            Err(fault) => Err(fault),
        }
    }

    pub fn execute_instruction<FI, FO>(
        &mut self,
        get_input: &mut FI,
        do_output: &mut FO,
    ) -> Result<CpuStatus, CpuFault>
    where
        FI: FnMut() -> Result<Word, InputOutputError>,
        FO: FnMut(Word) -> Result<(), InputOutputError>,
    {
        let instruction = self.ram.fetch(self.pc)?;
        let decoded = decode(instruction, self.pc)?;
        //println!("executing at {}: {:?}", &self.pc, &decoded);
        let (state, next_pc) = match decoded.op {
            Opcode::Add => {
                self.execute_arithmetic_instruction(&decoded.addressing_modes, add)?;

                (CpuStatus::Run, self.pc.checked_add(&Word(4_i64))?)
            }
            Opcode::Multiply => {
                self.execute_arithmetic_instruction(&decoded.addressing_modes, mul)?;
                (CpuStatus::Run, self.pc.checked_add(&Word(4_i64))?)
            }
            Opcode::Read => match get_input() {
                Ok(input) => {
                    //println!("input word was {}", &input.0);
                    self.put(&decoded.addressing_modes, 1, input)?;
                    (CpuStatus::Run, self.pc.checked_add(&Word(2_i64))?)
                }
                Err(e) => {
                    return Err(CpuFault::IOError(e));
                }
            },
            Opcode::Write => {
                let output = self.get(&decoded.addressing_modes, 1)?;
                match do_output(output) {
                    Ok(()) => (CpuStatus::Run, self.pc.checked_add(&Word(2_i64))?),
                    Err(e) => {
                        return Err(CpuFault::IOError(e));
                    }
                }
            }
            Opcode::JumpTrue => {
                let val: Word = self.get(&decoded.addressing_modes, 1)?;
                let next_pc = if val.0 != 0 {
                    self.get(&decoded.addressing_modes, 2)?
                } else {
                    self.pc.checked_add(&Word(3_i64))?
                };
                (CpuStatus::Run, next_pc)
            }
            Opcode::JumpFalse => {
                let val: Word = self.get(&decoded.addressing_modes, 1)?;
                let next_pc = if val.0 == 0 {
                    self.get(&decoded.addressing_modes, 2)?
                } else {
                    self.pc.checked_add(&Word(3_i64))?
                };
                (CpuStatus::Run, next_pc)
            }
            Opcode::CmpLess => {
                let less: bool = self.get(&decoded.addressing_modes, 1)?
                    < self.get(&decoded.addressing_modes, 2)?;
                self.put(&decoded.addressing_modes, 3, Word(if less { 1 } else { 0 }))?;
                (CpuStatus::Run, self.pc.checked_add(&Word(4_i64))?)
            }
            Opcode::CmpEq => {
                let left: Word = self.get(&decoded.addressing_modes, 1)?;
                let right: Word = self.get(&decoded.addressing_modes, 2)?;
                let equal: bool = left == right;
                //println!("CmpEq: {}=={}: {}", &left, &right, equal);
                self.put(
                    &decoded.addressing_modes,
                    3,
                    Word(if equal { 1 } else { 0 }),
                )?;
                (CpuStatus::Run, self.pc.checked_add(&Word(4_i64))?)
            }
            Opcode::DeltaRelBase => {
                let base = self.get(&decoded.addressing_modes, 1)?;
                self.update_relative_base(base)?;
                (CpuStatus::Run, self.pc.checked_add(&Word(2_i64))?)
            }
            Opcode::Stop => (CpuStatus::Halt, self.pc),
        };
        self.pc = next_pc;
        Ok(state)
    }

    fn get(
        &mut self,
        modes: &[AddressingMode; NUM_PARAMS],
        index: usize,
    ) -> Result<Word, CpuFault> {
        assert!(matches!(index, 1 | 2 | 3));
        let fetch_loc: Word = self.pc.checked_add_usize(&index)?;
        match modes[index] {
            AddressingMode::POSITIONAL => Ok(self.ram.fetch(self.ram.fetch(fetch_loc)?)?),
            AddressingMode::IMMEDIATE => Ok(self.ram.fetch(fetch_loc)?),
            AddressingMode::RELATIVE => {
                let base: Word = Word(self.relative_base);
                let offset = self.ram.fetch(fetch_loc)?;
                let rel_loc: Word = offset.checked_add(&base)?;
                Ok(self.ram.fetch(rel_loc)?)
            }
        }
    }

    fn put(
        &mut self,
        modes: &[AddressingMode; NUM_PARAMS],
        index: usize,
        value: Word,
    ) -> Result<(), CpuFault> {
        assert!(matches!(index, 1 | 2 | 3));
        let fetch_loc = self.pc.checked_add_usize(&index)?;
        let store_loc = match modes[index] {
            AddressingMode::POSITIONAL => self.ram.fetch(fetch_loc)?,
            AddressingMode::RELATIVE => self
                .ram
                .fetch(fetch_loc)?
                .checked_add(&Word(self.relative_base))?,
            AddressingMode::IMMEDIATE => {
                return Err(CpuFault::AddressingModeNotValidInContext);
            }
        };
        self.ram.store(store_loc, value)?;
        Ok(())
    }

    pub fn ram(&self) -> Vec<Word> {
        let mut result = Vec::new();
        self.ram.dump(&mut result);
        result
    }

    pub fn load(&mut self, base: Word, content: &[Word]) -> Result<(), CpuFault> {
        self.ram.load(base, content)
    }

    pub fn run_with_io<FI, FO>(
        &mut self,
        get_input: &mut FI,
        do_output: &mut FO,
    ) -> Result<(), CpuFault>
    where
        FI: FnMut() -> Result<Word, InputOutputError>,
        FO: FnMut(Word) -> Result<(), InputOutputError>,
    {
        while self.execute_instruction(get_input, do_output)? == CpuStatus::Run {
            // No need to do anything in the body.
        }
        Ok(())
    }

    pub fn run_with_fixed_input<FO>(
        &mut self,
        fixed_input: &[Word],
        do_output: &mut FO,
    ) -> Result<(), CpuFault>
    where
        FO: FnMut(Word) -> Result<(), InputOutputError>,
    {
        let mut it = fixed_input.iter();
        let mut get_input = || -> Result<Word, InputOutputError> {
            if let Some(val) = it.next() {
                Ok(*val)
            } else {
                Err(InputOutputError::NoInput) // no input available
            }
        };
        loop {
            match self.execute_instruction(&mut get_input, do_output) {
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
    let mut get_input = || -> Result<Word, InputOutputError> {
        if let Some(val) = it.next() {
            Ok(*val)
        } else {
            Err(InputOutputError::NoInput)
        }
    };
    let mut output = Vec::new();
    let mut do_output = |w: Word| -> Result<(), InputOutputError> {
        output.push(w);
        Ok(())
    };

    let mut cpu = Processor::new(Word(0));
    cpu.load(Word(0), &w_program)
        .expect("0 should be a valid load address");
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
}

#[test]
fn test_quine() {
    // This test case is given as an example in day 9.
    let quine = &[
        109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
    ];
    check_program(quine, &[], quine, quine);
}

#[derive(Debug)]
pub enum ProgramLoadError {
    ReadFailed(std::io::Error),
    BadWord(String, ParseIntError),
}

impl Display for ProgramLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramLoadError::ReadFailed(e) => {
                write!(f, "failed to read program: {}", e)
            }
            ProgramLoadError::BadWord(s, e) => {
                write!(f, "program contained invalid word '{}': {}", s, e)
            }
        }
    }
}

impl std::error::Error for ProgramLoadError {}

pub fn read_program_from_stdin() -> Result<Vec<Word>, ProgramLoadError> {
    let mut words: Vec<Word> = Vec::new();
    for input_element in io::BufReader::new(io::stdin()).lines() {
        match input_element {
            Err(e) => {
                return Err(ProgramLoadError::ReadFailed(e));
            }
            Ok(line) => {
                for field in line.trim().split(',') {
                    match field.parse::<i64>() {
                        Ok(n) => {
                            words.push(Word(n));
                        }
                        Err(e) => {
                            return Err(ProgramLoadError::BadWord(field.to_string(), e));
                        }
                    }
                }
            }
        }
    }
    Ok(words)
}
