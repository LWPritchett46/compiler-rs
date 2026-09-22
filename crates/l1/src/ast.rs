use std::fs::File;
use std::io::Write;

pub enum ArithOp {
    ADD,
    SUB,
    MUL,
    AND,
}

pub enum CompareOp {
    LT,
    LEQ,
    EQ,
}

pub enum ShiftOp {
    SHL,
    SHR,
}

pub struct Context {
    pub args: u64,
    pub locals: u64
}

pub trait Item {
    fn format(&self) -> String;
    fn format8(&self) -> String {
        self.format()
    }
}

pub struct Number {
    pub n: i64
}

impl Item for Number {
    fn format(&self) -> String {
        format!("${}", self.n)
    }
}

pub struct Label {
    name: String
}

impl Label {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Item for Label {
    fn format(&self) -> String {
        format!("_{}", &self.name[1..])
    }
}

pub struct Memory {
    base: Box<dyn Item>,
    offset: i64
}

impl Memory {
    pub fn new(base: Box<dyn Item>, offset: i64) -> Self {
        Self {base, offset}
    }
}

impl Item for Memory {
    fn format (&self) -> String {
        format!("{}({})", self.offset, self.base.format())
    }
}

#[allow(non_camel_case_types)]
pub enum RegisterID {
    rax, rbx, rcx, rdx,
    rsi, rdi, rbp, rsp,
    r8,  r9,  r10, r11,
    r12, r13, r14, r15
}

pub struct Register {
    id: RegisterID
}

impl Register {
    pub fn new(id: RegisterID) -> Self {
        Self { id }
    }
}

impl Item for Register {
    fn format(&self) -> String {
        use crate::ast::RegisterID::*;
        match self.id {
            rax => "%rax",
            rbx => "%rbx",
            rcx => "%rcx",
            rdx => "%rdx",
            rsi => "%rsi",
            rdi => "%rdi",
            rbp => "%rbp",
            rsp => "%rsp",
            r8  => "%r8",
            r9  => "%r9",
            r10 => "%r10",
            r11 => "%r11",
            r12 => "%r12",
            r13 => "%r13",
            r14 => "%r14",
            r15 => "%r15",
        }.to_string()
    }

    fn format8(&self) -> String {
        use crate::ast::RegisterID::*;
        match self.id {
            rax => "%al",
            rbx => "%bl",
            rcx => "%cl",
            rdx => "%dl",
            rsi => "%sil",
            rdi => "%dil",
            rbp => "%bpl",
            rsp => "%spl",
            r8  => "%r8b",
            r9  => "%r9b",
            r10 => "%r10b",
            r11 => "%r11b",
            r12 => "%r12b",
            r13 => "%r13b",
            r14 => "%r14b",
            r15 => "%r15b",
        }.to_string()
    }
}

pub trait Instruction {
    fn generate(&self, file: &mut File, ctxt: Context) -> std::io::Result<()>;
}

pub struct InstRet;

impl Instruction for InstRet {
    fn generate(&self, file: &mut File, ctxt: Context) -> std::io::Result<()> {
        let offset = (ctxt.args.saturating_sub(6) + ctxt.locals) * 8;

        if offset != 0 {
            writeln!(file, "  addq ${offset}, %rsp")?;
        }

        writeln!(file, "  retq")?;

        Ok(())
    }
}

pub struct InstAssign {
    dst: Box<dyn Item>,
    src: Box<dyn Item>,
}

impl InstAssign {
    pub fn new(dst: Box<dyn Item>, src: Box<dyn Item>) -> Self {
        Self {dst, src}
    }
}

impl Instruction for InstAssign {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        let src_str: String = self.src.format().chars()
            .enumerate()
            .map(|(idx, val)| if idx == 0 && val == '_' { '$' } else { val})
            .collect();

        let dst_str = self.dst.format();

        writeln!(file, "  movq {src_str}, {dst_str}")?;

        Ok(())
    }
}

pub struct InstStore {
    mem: Box<dyn Item>,
    src: Box<dyn Item>,
}

impl InstStore {
    pub fn new(mem: Box<dyn Item>, src: Box<dyn Item>) -> Self {
        Self {mem, src}
    }
}

impl Instruction for InstStore {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        let src_str: String = self.src.format().chars()
            .enumerate()
            .map(|(idx, val)| if idx == 0 && val == '_' { '$' } else { val})
            .collect();

        let mem_str = self.mem.format();

        writeln!(file, "  movq {src_str}, {mem_str}")?;

        Ok(())
    }
}

pub struct InstLoad {
    dst: Box<dyn Item>,
    mem: Box<dyn Item>,
}

impl InstLoad {
    pub fn new(dst: Box<dyn Item>, mem: Box<dyn Item>) -> Self {
        Self {dst, mem}
    }
}

impl Instruction for InstLoad {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        let mem_str = self.mem.format();
        let dst_str = self.dst.format();

        writeln!(file, "  movq {mem_str}, {dst_str}")?;

        Ok(())
    }
}

struct InstArith {
    lhs: Box<dyn Item>,
    op: ArithOp,
    rhs: Box<dyn Item>,
}

impl Instruction for InstArith {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        use crate::ast::ArithOp::*;

        let lhs_str = self.lhs.format();
        let rhs_str = self.rhs.format();
        let op_str = match self.op {
            ADD => "addq",
            SUB => "subq",
            MUL => "imulq",
            AND => "andq",
        };

        writeln!(file, "  {op_str} {rhs_str}, {lhs_str}")?;

        Ok(())
    }
}

struct InstInc {
    val: Box<dyn Item>,
}

impl Instruction for InstInc {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        let val_str = self.val.format();

        writeln!(file, "  inc {val_str}")?;

        Ok(())
    }
}

struct InstDec {
    val: Box<dyn Item>,
}

impl Instruction for InstDec {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        let val_str = self.val.format();

        writeln!(file, "  dec {val_str}")?;

        Ok(())
    }
}

struct InstMemRhs {
    lhs: Box<dyn Item>,
    op: ArithOp,
    mem: Box<dyn Item>
}

impl Instruction for InstMemRhs {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        use crate::ast::ArithOp::*;
        let lhs_str = self.lhs.format();
        let mem_str = self.mem.format();
        let op_str = match self.op {
            ADD => "addq",
            SUB => "subq",
            MUL => "imulq",
            AND => "andq"
        };

        writeln!(file, "  {op_str} {mem_str}, {lhs_str}")?;

        Ok(())
    }
}

struct InstMemLhs {
    mem: Box<dyn Item>,
    op: ArithOp,
    rhs: Box<dyn Item>,
}

struct InstCompAssign {
    dst: Box<dyn Item>,
    lhs: Box<dyn Item>,
    op: CompareOp,
    rhs: Box<dyn Item>,
}

impl Instruction for InstCompAssign {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        use crate::ast::CompareOp::*;

        let dst_str = self.dst.format();
        let dst_8 = self.dst.format8();
        let lhs_str = self.lhs.format();
        let rhs_str = self.rhs.format();

        match (lhs_str.starts_with('$'), rhs_str.starts_with('$')) {
            (true, true) => {
                let lhs_val: u64 = lhs_str[1..].parse().unwrap();
                let rhs_val: u64 = rhs_str[1..].parse().unwrap();
        
                let result = match self.op {
                    EQ => lhs_val == rhs_val,
                    LEQ => lhs_val <= rhs_val,
                    LT => lhs_val < rhs_val
                };

                writeln!(file, "  movq ${}, {}", result as u8, dst_str)?;
            },
            (true, false) => {
                writeln!(file, "  cmpq {lhs_str}, {rhs_str}")?;

                match self.op {
                    EQ => writeln!(file, "  sete {dst_8}")?,
                    LEQ => writeln!(file, "  setge {dst_8}")?,
                    LT => writeln!(file, "  setg {dst_8}")?
                }

                writeln!(file, "  movzbq {dst_8}, {dst_str}")?;
            },
            (false, _) => {
                writeln!(file, "  cmpq {rhs_str}, {lhs_str}")?;

                match self.op {
                    EQ => writeln!(file, "  sete {dst_8}")?,
                    LEQ => writeln!(file, "  setle {dst_8}")?,
                    LT => writeln!(file, "  setl {dst_8}")?
                }

                writeln!(file, "  movzbq {dst_8}, {dst_str}")?;
            }
        }

        Ok(())
    }
}

struct InstLabel {
    lbl: Box<dyn Item>,
}

impl Instruction for InstLabel {
    fn generate(&self, file: &mut File, _ctxt: Context) -> std::io::Result<()> {
        let lbl_str = self.lbl.format();

        writeln!(file, ":{lbl_str}");

        Ok(())
    }
}

pub struct Function {
    name: String,
    args: i64,
    locals: i64,
    insts: Vec<Box<dyn Instruction>>
}

impl Function {
    pub fn new(name: String, args: i64, locals: i64, insts: Vec<Box<dyn Instruction>>) -> Self {
        Self { name, args, locals, insts }
    }
}

pub struct Program {
    pub entry_point: String,
    pub funcs: Vec<Function>
}