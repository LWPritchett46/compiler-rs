use crate::ast::*;

peg::parser! {
    grammar l1_parser() for str {
        pub rule program() -> Program
            = "(" _
              entry_point:func_name() _ "\n"
              funcs:(_ f:function() _ "\n" { f })+
              _
              ")"
        {
            Program {
                entry_point,
                funcs
            }
        }

        rule function() -> Function
            = "(" _
              name:func_name() _ "\n"
              args:number() __ locals:number() _ "\n"
              insts:(_ i:instruction() _ "\n" { i })+
              ")"

        {
            Function::new(
                name,
                args,
                locals,
                insts
            )
        }

        rule instruction() -> Box<dyn Instruction>
            = inst_assign()
            / inst_store()
            / inst_load()

        rule inst_assign() -> Box<dyn Instruction>
            = dst:item_writable() _ "<-" _ src:item_source()
        {
            Box::new(InstAssign::new(dst, src))
        }

        rule inst_store() -> Box<dyn Instruction>
            = mem:item_memory() _ "<-" _ src:item_source()
        {
            Box::new(InstStore::new(mem, src))
        }

        rule inst_load() -> Box<dyn Instruction>
            = dst:item_writable() _ "<-" _ mem:item_memory()
        {
            Box::new(InstLoad::new(dst, mem))
        }

        rule item_memory() -> Box<dyn Item>
            = "mem" _ base:item_register() _ offset:number()
        {
            Box::new(Memory::new(base, offset))
        }

        rule item_source() -> Box<dyn Item>
            = item_label()
            / item_trivial()
            / item_func()

        rule item_trivial() -> Box<dyn Item>
            = item_number()
            / item_register()

        rule item_register() -> Box<dyn Item>
            = item_writable()
            / "rsp" {Box::new(Register::new(RegisterID::rsp))}

        rule item_writable() -> Box<dyn Item>
            = item_argument()
            / "rax" {Box::new(Register::new(RegisterID::rax))}
            / "rbx" {Box::new(Register::new(RegisterID::rbx))}
            / "rbp" {Box::new(Register::new(RegisterID::rbp))}

        rule item_argument() -> Box<dyn Item>
            = "rdi" {Box::new(Register::new(RegisterID::rdi))}
            / "rsi" {Box::new(Register::new(RegisterID::rsi))}
            / "rdx" {Box::new(Register::new(RegisterID::rdx))}
            / item_shift()
            / "r8" {Box::new(Register::new(RegisterID::r8))}
            / "r9" {Box::new(Register::new(RegisterID::r9))}

        rule item_shift() -> Box<dyn Item>
            = "rcx" {Box::new(Register::new(RegisterID::rcx))}


        rule item_number() -> Box<dyn Item>
            = n:number()
        {
            Box::new(Number { n })
        }

        rule item_func() -> Box<dyn Item>
            = name:func_name()
        {
            Box::new(Label::new(name))
        }

        rule item_label() -> Box<dyn Item>
            = name:$("$" identifier())
        {
            Box::new(Label::new(name.to_string()))
        }

        rule number() -> i64
            = n:$("-"? ['0'..='9']+)
        {
            n.parse().unwrap()
        }

        rule func_name() -> String
            = s:$("@" identifier())
        {
            s.to_string()
        }

        rule identifier() -> String
            = s:$(
                ['a'..='z' | 'A'..='Z' | '_']
                ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*
            )
        {
            s.to_string()
        }

        rule comment() = "//" [^'\n']* "\n"

        rule __ = [' ' | '\t' | '\r' | '\n' ]+ / comment()
        rule _ = [' ' | '\t' | '\r' | '\n' ]*
    }
}