pub mod runtime {
    use std::ops::Add;
    use std::ops::Div;
    use std::ops::Mul;
    use std::ops::Sub;
    use std::vec;

    use colored::Colorize;

    use crate::runtime::runtime_types;

    use super::runtime_error::*;
    use super::runtime_types::*;

    impl Context {
        pub fn new() -> Self {
            Self {
                stack: vec![],
                call_stack: [CallStack {
                    end: 0,
                    code_ptr: 0,
                    reg_freeze: [Types::Null; FREEZED_REG_SIZE],
                    pointers_len: 0,
                }; CALL_STACK_SIZE],
                registers: [Types::Null; REGISTER_SIZE],
                code: vec![],
                code_ptr: 0,
                heap: vec![],
                garbage: vec![],
                stack_ptr: 0,
                non_primitives: vec![],
                traits: vec![],
                break_code: None,
                catches: Catches {
                    catches_ptr: 0,
                    cache: [Catch {
                        code_ptr: 0,
                        id: None,
                        cs_ptr: 0,
                    }; CALL_STACK_SIZE],
                },
                exit_code: ExitCodes::End,
                string_arena: vec![],
            }
        }
        pub fn run(&mut self) -> bool {
            while self.read_line() {}
            return true;
        }
        fn read_line(&mut self) -> bool {
            macro_rules! operation {
                (ptr, $operand: ident, $num1: ident, bool) => {
                    if let Types::Pointer(num2, _) = self.registers[GENERAL_REG2] {
                        self.registers[GENERAL_REG1] = Types::Bool($num1.$operand(&num2));
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.registers[GENERAL_REG1],
                            self.registers[GENERAL_REG2],
                            self.code[self.code_ptr],
                        ));
                    }
                };
                ($type: tt, $operand: ident, $num1: ident, bool) => {
                    if let Types::$type(num2) = self.registers[GENERAL_REG2] {
                        self.registers[GENERAL_REG1] = Types::Bool($num1.$operand(&num2));
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.registers[GENERAL_REG1],
                            self.registers[GENERAL_REG2],
                            self.code[self.code_ptr],
                        ));
                    }
                };
                ($type: tt, $operand: ident, $num1: ident) => {
                    if let Types::$type(num2) = self.registers[GENERAL_REG2] {
                        self.registers[GENERAL_REG1] = Types::$type($num1.$operand(num2));
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.registers[GENERAL_REG1],
                            self.registers[GENERAL_REG2],
                            self.code[self.code_ptr],
                        ));
                    }
                };
                ($type: tt, %, $num1: ident) => {
                    if let Types::$type(num2) = self.registers[GENERAL_REG2] {
                        self.registers[GENERAL_REG1] = Types::$type($num1 % num2);
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.registers[GENERAL_REG1],
                            self.registers[GENERAL_REG2],
                            self.code[self.code_ptr],
                        ));
                    }
                };
            }
            use Instructions::*;
            match self.code[self.code_ptr] {
                Wr(stack_offset, register) => {
                    let end = self.stack_end();
                    self.stack[end - stack_offset] = self.registers[register];
                    self.next_line();
                }
                Rd(stack_offset, reg) => {
                    let end = self.stack_end();
                    self.registers[reg] = self.stack[end - stack_offset];
                    self.next_line();
                }
                Wrp(value_reg) => {
                    if let Types::Pointer(u_size, kind) = self.registers[POINTER_REG] {
                        match kind {
                            PointerTypes::Stack => {
                                self.stack[u_size] = self.registers[value_reg];
                            }
                            PointerTypes::Heap(loc) => {
                                self.heap[u_size][loc] = self.registers[value_reg];
                            }
                            PointerTypes::Object => {
                                return self.panic_rt(ErrTypes::Expected(
                                    Types::Pointer(0, PointerTypes::Heap(0)),
                                    self.registers[POINTER_REG],
                                ));
                            }
                            PointerTypes::String => {
                                if let Types::Pointer(dest, PointerTypes::String) =
                                    self.registers[value_reg]
                                {
                                    self.str_copy_from(u_size, dest)
                                } else {
                                    return self.panic_rt(ErrTypes::Expected(
                                        Types::Pointer(0, PointerTypes::String),
                                        self.registers[value_reg],
                                    ));
                                }
                            }
                            PointerTypes::Char(loc) => {
                                if let Types::Char(chr) = self.registers[value_reg] {
                                    self.string_arena[u_size][loc] = chr
                                } else {
                                    return self.panic_rt(ErrTypes::Expected(
                                        Types::Char('a'),
                                        self.registers[value_reg],
                                    ));
                                }
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::Pointer(0, PointerTypes::Heap(0)),
                            self.registers[POINTER_REG],
                        ));
                    }
                    self.next_line();
                }
                Rdp(cash_reg) => {
                    if let Types::Pointer(u_size, kind) = self.registers[POINTER_REG] {
                        match kind {
                            PointerTypes::Stack => {
                                self.registers[cash_reg] = self.stack[u_size];
                            }
                            PointerTypes::Heap(idx) => {
                                self.registers[cash_reg] = self.heap[u_size][idx];
                            }
                            PointerTypes::Object => {
                                return self.panic_rt(ErrTypes::InvalidType(
                                    self.registers[POINTER_REG],
                                    Types::Pointer(0, PointerTypes::Heap(0)),
                                ));
                            }
                            PointerTypes::String => {
                                return self.panic_rt(ErrTypes::InvalidType(
                                    self.registers[POINTER_REG],
                                    Types::Pointer(0, PointerTypes::Char(0)),
                                ));
                            }
                            PointerTypes::Char(idx) => {
                                self.registers[cash_reg] =
                                    Types::Char(self.string_arena[u_size][idx]);
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::InvalidType(
                            self.registers[POINTER_REG],
                            Types::Pointer(0, PointerTypes::Heap(0)),
                        ));
                    }
                    self.next_line();
                }
                Rdc(stack_pos, reg) => {
                    self.registers[reg] = self.stack[stack_pos];
                    self.next_line();
                }
                Ptr(stack_offset) => {
                    self.registers[GENERAL_REG1] =
                        Types::Pointer(self.stack_end() - stack_offset, PointerTypes::Stack);
                    self.next_line();
                }
                Idx(index_reg) => {
                    if let Types::Pointer(u_size, kind) = self.registers[POINTER_REG] {
                        if let Types::Usize(index) = self.registers[index_reg] {
                            match kind {
                                PointerTypes::Object => {
                                    self.registers[POINTER_REG] =
                                        Types::Pointer(u_size, PointerTypes::Heap(index));
                                }
                                PointerTypes::Stack => {
                                    self.registers[POINTER_REG] =
                                        Types::Pointer(u_size + index, PointerTypes::Stack);
                                }
                                PointerTypes::Heap(_) => {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.registers[POINTER_REG],
                                        self.code[self.code_ptr],
                                    ));
                                }
                                PointerTypes::Char(_) => {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.registers[POINTER_REG],
                                        self.code[self.code_ptr],
                                    ));
                                }
                                PointerTypes::String => {
                                    self.registers[POINTER_REG] =
                                        Types::Pointer(u_size, PointerTypes::Char(index));
                                }
                            }
                        } else {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[POINTER_REG],
                                self.code[self.code_ptr],
                            ));
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[POINTER_REG],
                            self.code[self.code_ptr],
                        ));
                    }
                    self.next_line();
                }
                IdxK(index) => {
                    if let Types::Pointer(u_size, kind) = self.registers[POINTER_REG] {
                        match kind {
                            PointerTypes::Object => {
                                self.registers[POINTER_REG] =
                                    Types::Pointer(u_size, PointerTypes::Heap(index));
                            }
                            PointerTypes::Stack => {
                                self.registers[POINTER_REG] =
                                    Types::Pointer(u_size + index, PointerTypes::Stack);
                            }
                            PointerTypes::Heap(_) => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.registers[POINTER_REG],
                                    self.code[self.code_ptr],
                                ));
                            }
                            PointerTypes::Char(_) => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.registers[POINTER_REG],
                                    self.code[self.code_ptr],
                                ));
                            }
                            PointerTypes::String => {
                                self.registers[POINTER_REG] =
                                    Types::Pointer(u_size, PointerTypes::Char(index));
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[POINTER_REG],
                            self.code[self.code_ptr],
                        ));
                    }
                }
                Alc(size_reg) => {
                    if let Types::Usize(size) = self.registers[size_reg] {
                        self.registers[POINTER_REG] =
                            Types::Pointer(self.allocate_obj(size), PointerTypes::Object);
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::Usize(0),
                            self.registers[size_reg],
                        ));
                    }
                    self.next_line();
                }
                AlcS(size) => {
                    self.registers[POINTER_REG] =
                        Types::Pointer(self.allocate_obj(size), PointerTypes::Object);
                    self.next_line();
                }
                RAlc(size_reg) => {
                    if let Types::Pointer(u_size, ptr_type) = self.registers[POINTER_REG] {
                        match ptr_type {
                            PointerTypes::Object => {
                                if let Types::Usize(new_size) = self.registers[size_reg] {
                                    self.resize_obj(u_size, new_size);
                                } else {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.registers[size_reg],
                                        self.code[self.code_ptr],
                                    ));
                                }
                            }
                            PointerTypes::String => {
                                if let Types::Usize(new_size) = self.registers[size_reg] {
                                    self.string_arena[u_size].resize(new_size, 0 as char);
                                } else {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.registers[size_reg],
                                        self.code[self.code_ptr],
                                    ));
                                }
                            }
                            _ => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.registers[POINTER_REG],
                                    self.code[self.code_ptr],
                                ))
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[POINTER_REG],
                            self.code[self.code_ptr],
                        ));
                    }
                    self.next_line();
                }
                Sweep => {
                    self.sweep();
                    self.next_line();
                }
                SweepUnoptimized => {
                    self.sweep_unoptimized();
                    self.next_line();
                }
                Goto(pos) => {
                    self.code_ptr = pos;
                }
                Jump(pos) => {
                    self.call_stack[self.stack_ptr].code_ptr = self.code_ptr;
                    self.code_ptr = pos;
                }
                Gotop => {
                    if let Types::CodePointer(u_size) = self.registers[CODE_PTR_REG] {
                        self.code_ptr = u_size
                    } else {
                        return self.panic_rt(ErrTypes::InvalidType(
                            self.registers[CODE_PTR_REG],
                            Types::CodePointer(0),
                        ));
                    }
                }
                Brnc(pos1, pos2) => {
                    if let Types::Bool(bool) = self.registers[GENERAL_REG1] {
                        self.code_ptr = if bool { pos1 } else { pos2 };
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[GENERAL_REG1],
                            self.code[self.code_ptr],
                        ));
                    }
                }
                Ret => {
                    self.code_ptr = self.call_stack[self.stack_ptr].code_ptr;
                    self.stack_ptr -= 1;
                    self.next_line();
                }
                Back => {
                    self.code_ptr = self.call_stack[self.stack_ptr].code_ptr;
                    self.next_line();
                }
                Ufrz => {
                    for i in 0..FREEZED_REG_SIZE {
                        self.registers[i] = self.call_stack[self.stack_ptr].reg_freeze[i]
                    }
                    self.next_line();
                }
                Res(size, pointers_len) => {
                    let end = self.stack_end() + size;
                    self.stack_ptr += 1;
                    if self.stack_ptr >= self.call_stack.len() {
                        if self.stack_ptr > self.call_stack.len() {
                            loop {
                                println!("Samik mel pravdu, ale tohle stejne nikdy neuvidis ;p");
                            }
                        }
                        return self.panic_rt(ErrTypes::StackOverflow);
                    }
                    self.call_stack[self.stack_ptr].end = end;
                    self.call_stack[self.stack_ptr].pointers_len = pointers_len;
                    if end > self.stack.len() {
                        self.stack.resize(end, Types::Null);
                    }
                    self.next_line();
                }
                Frz => {
                    self.call_stack[self.stack_ptr]
                        .reg_freeze
                        .clone_from_slice(&self.registers[..3]);
                    self.next_line();
                }
                Swap(reg1, reg2) => {
                    let temp = self.registers[reg1];
                    self.registers[reg1] = self.registers[reg2];
                    self.registers[reg2] = temp;
                    self.next_line();
                }
                Move(reg1, reg2) => {
                    self.registers[reg2] = self.registers[reg1];
                    self.next_line();
                }
                Add => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, add, num1),
                        Types::Float(num1) => operation!(Float, add, num1),
                        Types::Byte(num1) => operation!(Byte, add, num1),
                        Types::Usize(num1) => operation!(Usize, add, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Sub => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, sub, num1),
                        Types::Float(num1) => operation!(Float, sub, num1),
                        Types::Byte(num1) => operation!(Byte, sub, num1),
                        Types::Usize(num1) => operation!(Usize, sub, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Mul => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, mul, num1),
                        Types::Float(num1) => operation!(Float, mul, num1),
                        Types::Byte(num1) => operation!(Byte, mul, num1),
                        Types::Usize(num1) => operation!(Usize, mul, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Div => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, div, num1),
                        Types::Float(num1) => operation!(Float, div, num1),
                        Types::Byte(num1) => operation!(Byte, div, num1),
                        Types::Usize(num1) => operation!(Usize, div, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Mod => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, %, num1),
                        Types::Float(num1) => operation!(Float, %, num1),
                        Types::Byte(num1) => operation!(Byte, %, num1),
                        Types::Usize(num1) => operation!(Usize, %, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Equ => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, eq, num1, bool),
                        Types::Float(num1) => operation!(Float, eq, num1, bool),
                        Types::Byte(num1) => operation!(Byte, eq, num1, bool),
                        Types::Usize(num1) => operation!(Usize, eq, num1, bool),
                        Types::Pointer(num1, _) => operation!(ptr, eq, num1, bool),
                        Types::Bool(var1) => operation!(Bool, eq, var1, bool),
                        Types::Char(char1) => operation!(Char, eq, char1, bool),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Grt => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, gt, num1, bool),
                        Types::Float(num1) => operation!(Float, gt, num1, bool),
                        Types::Byte(num1) => operation!(Byte, gt, num1, bool),
                        Types::Usize(num1) => operation!(Usize, gt, num1, bool),
                        Types::Char(char1) => operation!(Char, gt, char1, bool),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Less => {
                    match self.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, lt, num1, bool),
                        Types::Float(num1) => operation!(Float, lt, num1, bool),
                        Types::Byte(num1) => operation!(Byte, lt, num1, bool),
                        Types::Usize(num1) => operation!(Usize, lt, num1, bool),
                        Types::Char(char1) => operation!(Char, lt, char1, bool),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                And => {
                    match self.registers[GENERAL_REG1] {
                        Types::Bool(var1) => {
                            if let Types::Bool(var2) = self.registers[GENERAL_REG2] {
                                self.registers[GENERAL_REG1] = Types::Bool(var1 && var2)
                            } else {
                                return self.panic_rt(ErrTypes::CrossTypeOperation(
                                    self.registers[GENERAL_REG1],
                                    self.registers[GENERAL_REG2],
                                    self.code[self.code_ptr],
                                ));
                            }
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Or => {
                    match self.registers[GENERAL_REG1] {
                        Types::Bool(var1) => {
                            if let Types::Bool(var2) = self.registers[GENERAL_REG2] {
                                self.registers[GENERAL_REG1] = Types::Bool(var1 || var2)
                            } else {
                                return self.panic_rt(ErrTypes::CrossTypeOperation(
                                    self.registers[GENERAL_REG1],
                                    self.registers[GENERAL_REG2],
                                    self.code[self.code_ptr],
                                ));
                            }
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Not => {
                    match self.registers[GENERAL_REG1] {
                        Types::Bool(var) => self.registers[GENERAL_REG1] = Types::Bool(!var),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Cal(_procedure, _args) => {}
                Mtd(trt, method) => {
                    self.call_stack[self.stack_ptr].code_ptr = self.code_ptr;
                    self.code_ptr = self.traits[trt][method];
                }
                End => {
                    return false;
                }
                Debug(reg) => {
                    println!("{:+}", self.registers[reg]);
                    self.next_line();
                }
                Len(_) => {
                    /*
                    if let Types::Pointer(u_size, kind) = self.registers[POINTER_REG] {
                        if let PointerTypes::HeapReg = kind {
                            if let Some(registry) = self.heap_reg_idx(u_size) {
                                self.registers[POINTER_REG] =
                                    Types::Usize(self.heap[registry].data.len())
                            } else {
                                return self.panic_rt(ErrTypes::PointerInBrokenState);
                            }
                        } else {
                            return self.panic_rt(ErrTypes::NotObject(self.registers[POINTER_REG]));
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[POINTER_REG],
                            self.code[self.code_ptr],
                        ));
                    }
                    self.next_line();*/
                    todo!();
                }
                CpRng(original, new, len) => {
                    let new_ptr = if let Types::Pointer(u_size, kind) = self.registers[new] {
                        (u_size, kind)
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[new],
                            CpRng(0, 0, 0),
                        ));
                    };
                    if let Types::Pointer(u_size, kind) = self.registers[original] {
                        for i in 0..len {
                            let value = match kind {
                                PointerTypes::Object => self.heap[u_size][i],
                                PointerTypes::String => Types::Char(self.string_arena[u_size][i]),
                                PointerTypes::Stack => self.stack[u_size + i],
                                PointerTypes::Heap(idx) => self.heap[u_size][i + idx],
                                PointerTypes::Char(idx) => {
                                    Types::Char(self.string_arena[u_size][i + idx])
                                }
                            };
                            match new_ptr.1 {
                                PointerTypes::Object => {
                                    self.heap[new_ptr.0][i] = value;
                                }
                                PointerTypes::String => {
                                    self.string_arena[new_ptr.0][i] = value.get_char();
                                }
                                PointerTypes::Stack => {
                                    self.stack[new_ptr.0 + i] = value;
                                }
                                PointerTypes::Heap(idx) => {
                                    self.heap[new_ptr.0][idx + i] = value;
                                }
                                PointerTypes::Char(idx) => {
                                    self.string_arena[new_ptr.0][idx + i] = value.get_char();
                                }
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[original],
                            CpRng(0, 0, 0),
                        ));
                    }
                }
                TRng(val, len) => {
                    let value = self.registers[val];
                    if let Types::Pointer(u_size, kind) = self.registers[POINTER_REG] {
                        for i in 0..len {
                            match kind {
                                PointerTypes::Object => {
                                    self.heap[u_size][i] = value;
                                }
                                PointerTypes::Stack => {
                                    self.stack[u_size + i] = value;
                                }
                                PointerTypes::Heap(idx) => {
                                    self.heap[u_size][i + idx] = value;
                                }
                                PointerTypes::Char(idx) => {
                                    self.string_arena[u_size][i + idx] = value.get_char();
                                }
                                PointerTypes::String => {
                                    self.string_arena[u_size][i] = value.get_char();
                                }
                            }
                        }
                    }
                }
                Type(reg1, reg2) => {
                    use std::mem::discriminant;
                    self.registers[reg2] = Types::Bool(
                        discriminant(&self.registers[reg1]) == discriminant(&self.registers[reg2]),
                    );
                    self.next_line();
                }
                NPType(np_reg, id) => {
                    if let Types::NonPrimitive(id_dyn) = self.registers[np_reg] {
                        self.registers[GENERAL_REG3] = Types::Bool(id == id_dyn);
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::NonPrimitive(0),
                            self.registers[np_reg],
                        ));
                    }
                }
                Cast(reg1, ttype) => {
                    if let Err(err) = Self::cast(&mut self.registers, reg1, ttype) {
                        return self.panic_rt(err);
                    }
                    self.next_line();
                }
                Break(code) => {
                    self.break_code = Some(code);
                    return false;
                }
                Catch => {
                    if let Err(err) = self.catches.push(runtime_types::Catch {
                        code_ptr: self.code_ptr,
                        id: None,
                        cs_ptr: self.stack_ptr,
                    }) {
                        return self.panic_rt(err);
                    }
                    self.next_line()
                }
                CatchId(id) => {
                    if let Err(err) = self.catches.push(runtime_types::Catch {
                        code_ptr: self.code_ptr,
                        id: Some(id),
                        cs_ptr: self.stack_ptr,
                    }) {
                        return self.panic_rt(err);
                    }
                    self.next_line()
                }
                DelCatch => {
                    self.catches.pop();
                    self.next_line()
                }
                StrCpy(reg) => {
                    if let Types::Pointer(u_size, PointerTypes::String) = self.registers[reg] {
                        self.registers[POINTER_REG] =
                            Types::Pointer(self.str_copy(u_size), PointerTypes::String);
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::Pointer(0, PointerTypes::String),
                            self.registers[reg],
                        ));
                    }
                    self.next_line();
                }
                StrNew => {
                    self.registers[POINTER_REG] =
                        Types::Pointer(self.str_new(), PointerTypes::String);
                    self.next_line();
                }
                StrCat(reg) => {
                    if let Types::Pointer(left, PointerTypes::String) = self.registers[POINTER_REG]
                    {
                        if let Types::Pointer(right, PointerTypes::String) = self.registers[reg] {
                            self.registers[POINTER_REG] =
                                Types::Pointer(self.str_concat(left, right), PointerTypes::String);
                        } else {
                            return self.panic_rt(ErrTypes::Expected(
                                Types::Pointer(0, PointerTypes::String),
                                self.registers[reg],
                            ));
                        }
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::Pointer(0, PointerTypes::String),
                            self.registers[reg],
                        ));
                    }
                }
                Panic => {
                    let mut i = self.catches.catches_ptr;
                    loop {
                        if i == 0 {
                            self.exit_code = ExitCodes::Exception;
                            return false;
                        }
                        i -= 1;
                        if let Some(n) = self.catches.cache[i].id {
                            if let Types::NonPrimitive(e_type) = self.registers[RETURN_REG] {
                                if n == e_type {
                                    self.code_ptr = self.catches.cache[i].code_ptr;
                                    self.stack_ptr = self.catches.cache[i].cs_ptr;
                                }
                            }
                            break;
                        } else {
                            self.code_ptr = self.catches.cache[i].code_ptr;
                            self.stack_ptr = self.catches.cache[i].cs_ptr;
                            break;
                        }
                    }
                    self.catches.truncate(i);
                    self.next_line();
                }
            }
            return true;
        }
        // allocator starts here
        fn allocate_obj(&mut self, size: usize) -> usize {
            let mut data = Vec::new();
            data.resize(size, Types::Null);
            if let Some(idx) = self.garbage.pop() {
                self.heap[idx] = data;
                return idx;
            }
            self.heap.push(data);
            self.heap.len() - 1
        }
        fn resize_obj(&mut self, heap_idx: usize, new_size: usize) {
            self.heap[heap_idx].resize(new_size, Types::Null)
        }
        /// GC
        pub fn sweep(&mut self) {
            let marked = self.mark();
            self.sweep_marked(marked);
        }
        pub fn sweep_unoptimized(&mut self) {
            let marked = self.mark_unoptimized();
            self.sweep_marked(marked);
        }
        fn sweep_marked(&mut self, marked: Vec<bool>) {
            if let Some(idx) = marked.iter().rposition(|x| !*x) {
                self.heap.truncate(idx + 1);
            } else {
                self.heap.clear();
                return;
            }
            for (i, mark) in marked.iter().enumerate() {
                if i == self.heap.len() {
                    return;
                }
                if *mark {
                    self.heap[i].clear();
                    if !self.garbage.contains(&i) {
                        self.garbage.push(i);
                    }
                }
            }
        }
        fn mark_unoptimized(&mut self) -> Vec<bool> {
            let mut marked = Vec::new();
            marked.resize(self.heap.len(), true);
            self.mark_registers(&mut marked);
            self.mark_range((0, self.stack.len()), &mut marked);
            marked
        }
        fn mark(&mut self) -> Vec<bool> {
            let mut call_stack_idx = 1;
            let mut marked = Vec::new();
            marked.resize(self.heap.len(), true);
            self.mark_registers(&mut marked);
            while call_stack_idx <= self.stack_ptr {
                let cs = self.call_stack[call_stack_idx];
                let prev_cs = self.call_stack[call_stack_idx - 1];
                self.mark_range((prev_cs.end, prev_cs.end + cs.pointers_len), &mut marked);
                call_stack_idx += 1;
            }
            marked
        }
        fn mark_obj(&mut self, obj_idx: usize, marked: &mut Vec<bool>) {
            if !marked[obj_idx] {
                return;
            }
            marked[obj_idx] = false;
            for idx in 0..self.heap[obj_idx].len() {
                let member = self.heap[obj_idx][idx];
                if let Types::Pointer(u_size, PointerTypes::Object) = member {
                    self.mark_obj(u_size, marked);
                } else if let Types::Pointer(u_size, PointerTypes::Heap(_)) = member {
                    self.mark_obj(u_size, marked);
                }
            }
        }
        fn mark_range(&mut self, range: (usize, usize), marked: &mut Vec<bool>) {
            for idx in range.0..range.1 {
                if let Types::Pointer(u_size, PointerTypes::Heap(_)) = self.stack[idx] {
                    self.mark_obj(u_size, marked);
                } else if let Types::Pointer(u_size, PointerTypes::Object) = self.stack[idx] {
                    self.mark_obj(u_size, marked);
                }
            }
        }
        fn mark_registers(&mut self, marked: &mut Vec<bool>) {
            for reg in self.registers {
                if let Types::Pointer(u_size, PointerTypes::Heap(_)) = reg {
                    self.mark_obj(u_size, marked);
                } else if let Types::Pointer(u_size, PointerTypes::Object) = reg {
                    self.mark_obj(u_size, marked);
                }
            }
        }
        fn stack_end(&self) -> usize {
            self.call_stack[self.stack_ptr].end
        }
        fn next_line(&mut self) {
            self.code_ptr += 1;
        }
        fn cast(registers: &mut Registers, reg1: usize, reg2: usize) -> Result<bool, ErrTypes> {
            match registers[reg1] {
                Types::Bool(bol) => match registers[reg2] {
                    Types::Byte(_) => {
                        registers[reg1] = if bol { Types::Byte(1) } else { Types::Byte(0) }
                    }
                    Types::Int(_) => {
                        registers[reg1] = if bol { Types::Int(1) } else { Types::Int(0) }
                    }
                    Types::Float(_) => {
                        registers[reg1] = if bol {
                            Types::Float(1f64)
                        } else {
                            Types::Float(0f64)
                        }
                    }
                    Types::Usize(_) => {
                        registers[reg1] = if bol {
                            Types::Usize(1)
                        } else {
                            Types::Usize(0)
                        }
                    }
                    Types::Char(_) => {
                        registers[reg1] = if bol {
                            Types::Char('1')
                        } else {
                            Types::Char('0')
                        }
                    }
                    _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
                },
                Types::Byte(num) => match registers[reg2] {
                    Types::Int(_) => registers[reg1] = Types::Int(num as i32),
                    Types::Float(_) => registers[reg1] = Types::Float(num as f64),
                    Types::Usize(_) => registers[reg1] = Types::Usize(num as usize),
                    Types::Char(_) => registers[reg1] = Types::Char(num as char),
                    Types::Bool(_) => {
                        registers[reg1] = if num == 0 {
                            Types::Bool(false)
                        } else {
                            Types::Bool(true)
                        }
                    }
                    _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
                },
                Types::Int(num) => match registers[reg2] {
                    Types::Byte(_) => registers[reg1] = Types::Byte(num as u8),
                    Types::Float(_) => registers[reg1] = Types::Float(num as f64),
                    Types::Usize(_) => registers[reg1] = Types::Usize(num as usize),
                    Types::Bool(_) => {
                        registers[reg1] = if num == 0 {
                            Types::Bool(false)
                        } else {
                            Types::Bool(true)
                        }
                    }
                    _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
                },
                Types::Float(num) => match registers[reg2] {
                    Types::Byte(_) => registers[reg1] = Types::Byte(num as u8),
                    Types::Int(_) => registers[reg1] = Types::Int(num as i32),
                    Types::Usize(_) => registers[reg1] = Types::Usize(num as usize),
                    Types::Bool(_) => {
                        registers[reg1] = if num == 0f64 {
                            Types::Bool(false)
                        } else {
                            Types::Bool(true)
                        }
                    }
                    _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
                },
                Types::Usize(num) => match registers[reg2] {
                    Types::Byte(_) => registers[reg1] = Types::Byte(num as u8),
                    Types::Int(_) => registers[reg1] = Types::Int(num as i32),
                    Types::Float(_) => registers[reg1] = Types::Float(num as f64),
                    Types::Bool(_) => {
                        registers[reg1] = if num == 0 {
                            Types::Bool(false)
                        } else {
                            Types::Bool(true)
                        }
                    }
                    _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
                },
                _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
            }
            Ok(true)
        }
        /// Creates a new empty string and returns the location of the string.
        pub fn str_new(&mut self) -> usize {
            self.string_arena.push(vec![]);
            self.string_arena.len() - 1
        }
        /// Creates a new string from a vector of characters and returns the location of the string.
        pub fn str_from(&mut self, str: Vec<char>) -> usize {
            self.string_arena.push(str);
            self.string_arena.len() - 1
        }
        /// Copies a string from one location to a new location and returns the new location.
        pub fn str_copy(&mut self, loc: usize) -> usize {
            self.string_arena.push(self.string_arena[loc].clone());
            self.string_arena.len() - 1
        }
        /// Copies a string from one location to another location.
        pub fn str_copy_from(&mut self, orig: usize, dest: usize) {
            self.string_arena[dest] = self.string_arena[orig].clone()
        }
        pub fn str_concat(&mut self, left: usize, right: usize) -> usize {
            let mut temp = self.string_arena[left].clone();
            temp.extend(self.string_arena[right].iter());
            self.string_arena.push(temp);
            self.string_arena.len() - 1
        }
        pub fn data_report(&self, runtime: Option<u128>) {
            use enable_ansi_support::enable_ansi_support;
            match enable_ansi_support() {
                Ok(_) => {
                    print!("\n");
                    println!("{}", "Post-process data report.".yellow());
                    if let Some(time) = runtime {
                        println!("\x1b[90mTotal run time: {} ms\x1b[0m", time);
                    }
                    println!("{} {:?}", "Heap:".magenta(), self.heap);
                    println!("{} {:?}", "Stack:".magenta(), self.stack);
                    println!("{} {:?}", "Registers:".magenta(), self.registers);
                }
                Err(_) => {
                    print!("\n");
                    println!("{}", "Post-process data report.");
                    if let Some(time) = runtime {
                        println!("Total run time: {} ms", time);
                    }
                    println!("{} {:?}", "Heap:", self.heap);
                    println!("{} {:?}", "Stack:", self.stack);
                    println!("{} {:?}", "Registers:", self.registers);
                }
            }
        }
        fn panic_rt(&mut self, kind: ErrTypes) -> bool {
            self.break_code = Some(self.code_ptr);
            print_message(&kind);
            self.exit_code = ExitCodes::Internal(kind);
            false
        }
    }
}

pub mod runtime_error {
    use super::runtime_types::*;
    #[derive(Debug)]
    pub enum ErrTypes {
        CrossTypeOperation(Types, Types, Instructions),
        WrongTypeOperation(Types, Instructions),
        InvalidType(Types, Types),
        Expected(Types, Types),
        ImplicitCast(Types, Types),
        StackOverflow,
        CatchOwerflow,
    }
    fn gen_message(header: String, line: Option<(usize, usize)>, err_no: u8) -> String {
        return if let Some(line) = line {
            //                    code                      header                      line     column
            format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mAt: line {}, column {}.\x1b[0m", line.0, line.1)
        } else {
            format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mLocation unspecified.\x1b[0m")
        };
    }
    pub fn print_message(kind: &ErrTypes) {
        let data = match &kind {
            ErrTypes::CrossTypeOperation(var1, var2, instr) => (
                format!("Operation '{instr}' failed: Cross-type operation {var1:+}, {var2:+}"),
                0,
            ),
            ErrTypes::WrongTypeOperation(var1, instr) => (
                format!("Operation '{instr}' failed: Wrong-type operation {var1:+}"),
                1,
            ),
            ErrTypes::InvalidType(typ, operation) => (
                format!("Invalid Type: {typ:#} must be of type '{operation:#}'"),
                2,
            ),
            ErrTypes::Expected(exp, found) => {
                (format!("Wrong type: Expected {exp:#}, found {found:#}"), 3)
            }
            ErrTypes::ImplicitCast(type1, type2) => (
                format!("Cast error: Can not implicitly cast type {type1:#} into type {type2:#}"),
                4,
            ),
            ErrTypes::StackOverflow => (format!("Stack overflow"), 5), // TODO: impl this
            ErrTypes::CatchOwerflow => (format!("Catch overflow"), 6),
        };
        let message = gen_message(data.0, None, data.1);
        println!("{message}");
    }
}

#[allow(unused)]
pub mod runtime_types {
    pub const CALL_STACK_SIZE: usize = 256;
    pub const FREEZED_REG_SIZE: usize = 3;
    pub type Registers = [Types; REGISTER_SIZE];
    pub const REGISTER_SIZE: usize = 6;
    pub const GENERAL_REG1: usize = 0;
    pub const GENERAL_REG2: usize = 1;
    pub const GENERAL_REG3: usize = 2;
    pub const POINTER_REG: usize = 3;
    pub const RETURN_REG: usize = 4;
    pub const CODE_PTR_REG: usize = 5;
    /// context for a single thread of execution (may include multiple threads in future updates)
    /// this is the main struct that holds all the data for the runtime
    pub struct Context {
        pub stack: Vec<Types>,
        pub call_stack: [CallStack; CALL_STACK_SIZE],
        pub stack_ptr: usize,
        pub registers: Registers,
        pub code: Vec<Instructions>,
        pub code_ptr: usize,
        pub garbage: Vec<usize>,
        pub heap: Vec<Vec<Types>>,
        pub string_arena: Vec<Vec<char>>,
        pub non_primitives: Vec<NonPrimitiveType>,
        pub traits: Vec<Trait>,
        pub break_code: Option<usize>,
        pub catches: Catches,
        pub exit_code: ExitCodes,
    }
    #[derive(Debug, Copy, Clone)]
    pub struct Catches {
        pub catches_ptr: usize,
        pub cache: [Catch; CALL_STACK_SIZE],
    }
    impl Catches {
        /// pushes a new catch to the stack
        pub fn push(&mut self, catch: Catch) -> Result<(), ErrTypes> {
            if self.catches_ptr == CALL_STACK_SIZE {
                return Err(ErrTypes::CatchOwerflow);
            }
            self.catches_ptr += 1;
            self.cache[self.catches_ptr] = catch;
            Ok(())
        }
        /// pops the last catch from the stack
        pub fn pop(&mut self) {
            self.catches_ptr -= 1;
        }
        /// truncates the stack to a given size
        pub fn truncate(&mut self, n: usize) {
            self.catches_ptr = n;
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct Catch {
        pub code_ptr: usize,
        pub cs_ptr: usize,
        pub id: Option<usize>,
    }
    /// indicates why program exited
    #[derive(Debug)]
    pub enum ExitCodes {
        /// program ended
        End,
        /// program run into user defined break and is expected to continue in the future
        Break(usize),
        /// an exception was thrown but never caught
        Exception,
        /// unrecoverable error occured (if you believe this is not meant to happen, contact me)
        Internal(runtime_error::ErrTypes),
    }
    /// a structure used to register data on heap
    #[derive(Clone, Debug)]
    pub struct HeapRegistry {
        pub idx: usize,
        pub generation: u8,
    }
    #[derive(Clone, Copy, Debug)]
    pub enum Types {
        Int(i32),
        Float(f64),
        Usize(usize),
        Char(char),
        Byte(u8),
        Bool(bool),
        Pointer(usize, PointerTypes),
        CodePointer(usize),
        Null,
        /// header for non-primitive types
        /// ID
        NonPrimitive(usize),
        // is a pointer to string arena
        //String(usize),
    }
    impl Types {
        /// may panic, so use this only if you are 100% certain that you got a character
        pub fn get_char(&self) -> char {
            if let Types::Char(chr) = self {
                return *chr;
            }
            panic!()
        }
    }
    #[derive(Clone, Copy, Debug)]
    pub enum NonPrimitiveTypes {
        Array,
        Struct,
    }
    #[derive(Debug)]
    pub struct NonPrimitiveType {
        pub name: String,
        pub kind: NonPrimitiveTypes,
        pub len: usize,
        pub pointers: usize,
    }
    pub type Trait = Vec<usize>;
    use std::{clone, fmt, rc::Rc, sync::Arc};

    use super::runtime_error::{self, ErrTypes};
    impl fmt::Display for Types {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if f.alternate() {
                match *self {
                    Types::Bool(_) => write!(f, "Bool"),
                    Types::Byte(_) => write!(f, "Byte"),
                    Types::Char(_) => write!(f, "Char"),
                    Types::CodePointer(_) => write!(f, "CodePointer"),
                    Types::Float(_) => write!(f, "Float"),
                    Types::Int(_) => write!(f, "Int"),
                    Types::Null => write!(f, "Null"),
                    Types::Pointer(_, _) => write!(f, "Pointer"),
                    Types::Usize(_) => write!(f, "Usize"),
                    Types::NonPrimitive(_) => write!(f, "Non-primitive"),
                }
            } else if f.sign_plus() {
                match *self {
                    Types::Bool(bol) => {
                        write!(f, "Bool<{bol}>")
                    }
                    Types::Byte(byte) => write!(f, "Byte<{byte}>"),
                    Types::Char(char) => write!(f, "Char<{char}>"),
                    Types::CodePointer(loc) => write!(f, "CodePointer<{loc}>"),
                    Types::Float(num) => write!(f, "Float<{num}>"),
                    Types::Int(num) => write!(f, "Int<{num}>"),
                    Types::Null => write!(f, "Null"),
                    Types::Pointer(loc, kind) => write!(f, "Pointer<{loc}, {kind}>"),
                    Types::Usize(num) => write!(f, "Usize<{num}>"),
                    Types::NonPrimitive(id) => write!(f, "Non-primitive<{id}>"),
                }
            } else {
                match *self {
                    Types::Bool(bol) => write!(f, "{bol}"),
                    Types::Byte(byte) => write!(f, "{byte}"),
                    Types::Char(char) => write!(f, "{char}"),
                    Types::CodePointer(loc) => write!(f, "{loc}"),
                    Types::Float(num) => write!(f, "{num}"),
                    Types::Int(num) => write!(f, "{num}"),
                    Types::Null => write!(f, "Null"),
                    Types::Pointer(loc, _) => write!(f, "{loc}"),
                    Types::Usize(num) => write!(f, "{num}"),
                    Types::NonPrimitive(id) => write!(f, "{id}"),
                }
            }
        }
    }
    impl fmt::Display for NonPrimitiveTypes {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                NonPrimitiveTypes::Array => write!(f, "Array"),
                NonPrimitiveTypes::Struct => write!(f, "Struct"),
            }
        }
    }
    /// runtime
    #[derive(Clone, Copy, Debug)]
    pub enum PointerTypes {
        /// location on stack
        ///
        /// expires out of scope
        Stack,
        /// object
        /// needs to be transformed into heap pointer
        /// with index(usize)
        ///
        /// never expires, GC may change value
        Object,
        /// location on heap
        ///
        /// may expire any time
        Heap(usize),
        /// String
        ///
        /// location in string arena
        /// never expires
        String,
        /// char
        ///
        /// location and index in string arena
        /// may expire any time
        Char(usize),
    }
    impl fmt::Display for PointerTypes {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                PointerTypes::Heap(n) => write!(f, "Heap({n})"),
                PointerTypes::Object => write!(f, "Object"),
                PointerTypes::Stack => write!(f, "Stack"),
                PointerTypes::String => write!(f, "String"),
                PointerTypes::Char(n) => write!(f, "Stack({n})"),
            }
        }
    }
    /// complete list of runtime instructions
    #[allow(unused)]
    #[derive(Clone, Copy, Debug)]
    pub enum Instructions {
        /// Debug: reg | prints value of reg(<reg>)
        Debug(usize),
        /// Write: stack_offset reg | moves value from reg(0) to stack(stack_end - <stack_offset>)
        Wr(usize, usize),
        /// Read: stack_offset reg | reads value from stack(stack_end - <stack_offset>) to its reg(<reg>)
        Rd(usize, usize),
        /// WritePointer: value_reg | moves value from reg(<value_reg>) to stack(<pointer>)
        Wrp(usize),
        /// ReadPointer: reg | reads value from reg(pointer_reg) to its reg(<reg>)
        Rdp(usize),
        /// ReadConstant: stack_pos reg | reads value from stack(<stack_pos>) to its reg(<reg>)
        Rdc(usize, usize),
        /// Pointer: stack_pos | stores pointer to stack(stack_end - <stack_offset>) in reg(0)
        Ptr(usize),
        /// Index: idx | gets pointer from reg(<pointer>) repairs it and adds reg(<idx>)
        Idx(usize),
        /// Allocate: size_reg pointers_len | reserves <size> on heap and stores location in registers(<reg>)
        Alc(usize),
        /// Reallocate: size_reg | resizes heap(<reg>) for <size>; additional space is filled with null
        RAlc(usize),
        /// Goto: pos | moves code_pointer to <pos>
        Goto(usize),
        /// GotoCodePtr: pos_reg | moves code pointer to reg(<reg>)
        Gotop,
        /// Branch: pos1 pos2 | if reg(0), goto <pos1> else goto <pos2>
        Brnc(usize, usize),
        /// Return: | moves code_pointer to the last position in callstack and moves callstack back
        Ret,
        /// Unfreeze | returns registers to their last freezed state
        Ufrz,
        /// Reserve: size | reserves <size> on stack and advances callstack, also saves number of pointers for faster memory sweeps
        Res(usize, usize),
        /// Swap: reg1 reg2   | swaps <reg1> and <reg2>
        Swap(usize, usize),
        /// Add | reg(0) is set to the result of operation: reg(0) + reg(1)
        Add,
        /// Subtract | reg(0) is set to the result of operation: reg(0) - reg(1)
        Sub,
        /// Multiply | reg(0) is set to the result of operation: reg(0) * reg(1)
        Mul,
        /// Divide | reg(0) is set to the result of operation: reg(0) / reg(1)
        Div,
        /// Modulus | reg(0) is set to the result of operation: reg(0) % reg(1)
        Mod,
        /// Equals | reg(0) is set to the result of operation: reg(0) = reg(1)
        Equ,
        /// Greater than | reg(0) is set to the result of operation: reg(0) > reg(1)
        Grt,
        /// Less than | reg(0) is set to the result of operation: reg(0) < reg(1)
        Less,
        /// And | reg(0) is set to the result of operation: reg(0) & reg(1)
        And,
        /// Or | reg(0) is set to the result of operation: reg(0) | reg(1)
        Or,
        /// Not | reg(0) is set to the result of operation: !reg(0)
        Not,
        /// Call | calls external <procedure>(program state, <args>) written in rust (for syscalls etc..)
        Cal(usize, usize),
        /// End              | terminates program
        End,
        //TODO: add to compiler
        /// Cast: reg1 reg2 | casts value of reg1 to the type of reg2 and stores in reg1
        Cast(usize, usize),
        /// Length: reg | sets reg to Usize(size of heap object)
        Len(usize),
        /// Type: val type | sets reg(type) to bool(typeof(val) == typeof(type))
        Type(usize, usize),
        /// Jump: pos | moves code_pointer to <pos> and saves current code ptr
        Jump(usize),
        /// Freeze | freezes registers on callstack
        Frz,
        /// Back | returns to last code ptr
        Back,
        /// Move: reg1 reg2 | moves value of reg1 to reg2
        Move(usize, usize),
        /// Sweep | sweeps memory, deallocating all unaccesable objects
        Sweep,
        /// Sweep unoptimized | sweeps memory, deallocating all unaccesable objects, this instruction is here only to help me test GC since it doesnt require any code structure
        SweepUnoptimized,
        /// Allocate size: size | allocates new object with size known at compile time and returns pointer to reg(0)
        AlcS(usize),
        /// Index known: index | indexing operation where index is known at compile time (generally for structures but can be also used for arrays or single values on heap)
        IdxK(usize),
        /// To range: val_reg len | takes pointer at reg(POINTER_REG) as a starting point and fills len to the right with value on reg(value_reg)
        TRng(usize, usize),
        /// Copy range: original_ptr new_ptr len | copies range starting at reg(original_ptr) with size len to reg(new_ptr)
        CpRng(usize, usize, usize),
        /// Break: code | program exits with a break code, indicating that it should be resumed at some point
        Break(usize),
        /// Method: trait method | calls method belonging trait
        Mtd(usize, usize),
        /// Panic | program enters panic mode, returning from all stacks until exception is caught
        Panic,
        /// Catch | catches an error and returns program to normal mode, cached if read in normal mode
        Catch,
        /// Catch ID: id | same as normal catch but responds only to exception with same id
        CatchId(usize),
        /// Delete catch | deletes one chatch instruction from cache
        DelCatch,
        /// Non-primitive type: np_reg ID | compares reg(np_reg).id asuming it belongs to Non-primitive type with ID
        NPType(usize, usize),
        /// String new | creates new string and stores pointer in reg(POINTER_REGISTER)
        StrNew,
        /// String copy: str_reg | copies string from reg(str_reg) to new string and stores pointer in reg(POINTER_REGISTER)
        StrCpy(usize),
        /// String concat: val_reg | creates new string {reg(POINTER_REGISTER) + reg(value_reg)} and stores pointer in reg(POINTER_REG)
        StrCat(usize),
    }
    impl fmt::Display for Instructions {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let str = match *self {
                Instructions::Add => "Addition",
                Instructions::Alc(_) => "Allocation",
                Instructions::AlcS(_) => "Allocation",
                Instructions::And => "And",
                Instructions::Brnc(_, _) => "Branch",
                Instructions::Cal(_, _) => "Call",
                Instructions::Debug(_) => "Debug",
                Instructions::Div => "Division",
                Instructions::End => "End",
                Instructions::Equ => "Equality",
                Instructions::Goto(_) => "GoTo",
                Instructions::Gotop => "GoToDyn",
                Instructions::Grt => "Greater",
                Instructions::Idx(_) => "Indexing",
                Instructions::IdxK(_) => "Indexing",
                Instructions::Less => "Lesser",
                Instructions::Mod => "Modulus",
                Instructions::Swap(_, _) => "Swap",
                Instructions::Mul => "Multiplication",
                Instructions::Not => "Not",
                Instructions::Or => "Or",
                Instructions::Ptr(_) => "StackPointer",
                Instructions::RAlc(_) => "Reallocation",
                Instructions::Ufrz => "Unfreeze",
                Instructions::Rd(_, _) => "Read",
                Instructions::Rdc(_, _) => "ReadConst",
                Instructions::Rdp(_) => "Dereference",
                Instructions::Res(_, _) => "Reserve",
                Instructions::Ret => "Return",
                Instructions::Sub => "Subtract",
                Instructions::Wr(_, _) => "Write",
                Instructions::Wrp(_) => "WriteRef",
                Instructions::Cast(_, _) => "Casting",
                Instructions::Len(_) => "Length",
                Instructions::Type(_, _) => "TypeOf",
                Instructions::Jump(_) => "Jump",
                Instructions::Frz => "Freeze",
                Instructions::Back => "Back",
                Instructions::Move(_, _) => "Move",
                Instructions::Sweep => "Sweep",
                Instructions::SweepUnoptimized => "SweepUnoptimized",
                Instructions::TRng(_, _) => "ToRange",
                Instructions::CpRng(_, _, _) => "CopyRange",
                Instructions::Mtd(_, _) => "Method",
                Instructions::Break(_) => "Break",
                Instructions::Panic => "Panic",
                Instructions::Catch => "Catch",
                Instructions::CatchId(_) => "Catch",
                Instructions::DelCatch => "DeleteCatch",
                Instructions::NPType(_, _) => "NonPrimitiveType",
                Instructions::StrNew => "StringNew",
                Instructions::StrCpy(_) => "StringCopy",
                Instructions::StrCat(_) => "StringConcat",
            };
            write!(f, "{str}")
        }
    }
    /// holds information of where to jump after function call ends
    #[derive(Clone, Copy, Debug)]
    pub struct CallStack {
        pub reg_freeze: [Types; FREEZED_REG_SIZE],
        pub end: usize,
        pub code_ptr: usize,
        pub pointers_len: usize,
    }
}
