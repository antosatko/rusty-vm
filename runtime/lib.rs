pub mod runtime {
    use std::ops::Add;
    use std::ops::Div;
    use std::ops::Mul;
    use std::ops::Sub;
    use std::vec;

    use runtime_error::*;
    use runtime_types::*;

    impl Context {
        pub fn new() -> Self {
            Self {
                memory: Memory {
                    stack: Stack {
                        data: vec![],
                        ptr: 0,
                        call_stack: [CallStack {
                            end: 0,
                            code_ptr: 0,
                            reg_freeze: [Types::Null; FREEZED_REG_SIZE],
                            pointers_len: 0,
                        }; CALL_STACK_SIZE],
                    },
                    registers: [Types::Null; REGISTER_SIZE],
                    heap: Heap {
                        data: vec![],
                        garbage: vec![],
                    },
                    strings: Strings {
                        pool: vec![],
                        garbage: vec![],
                    },
                    non_primitives: vec![],
                },
                code: Code {
                    data: vec![],
                    ptr: 0,
                },

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

                libs: vec![],
            }
        }
        pub fn run(&mut self) -> bool {
            while self.read_line() {}
            return true;
        }
        fn read_line(&mut self) -> bool {
            macro_rules! operation {
                (ptr, $operand: ident, $num1: ident, bool) => {
                    if let Types::Pointer(num2, _) = self.memory.registers[GENERAL_REG2] {
                        self.memory.registers[GENERAL_REG1] = Types::Bool($num1.$operand(&num2));
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.memory.registers[GENERAL_REG1],
                            self.memory.registers[GENERAL_REG2],
                            self.code.data[self.code.ptr],
                        ));
                    }
                };
                ($type: tt, $operand: ident, $num1: ident, bool) => {
                    if let Types::$type(num2) = self.memory.registers[GENERAL_REG2] {
                        self.memory.registers[GENERAL_REG1] = Types::Bool($num1.$operand(&num2));
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.memory.registers[GENERAL_REG1],
                            self.memory.registers[GENERAL_REG2],
                            self.code.data[self.code.ptr],
                        ));
                    }
                };
                ($type: tt, $operand: ident, $num1: ident) => {
                    if let Types::$type(num2) = self.memory.registers[GENERAL_REG2] {
                        self.memory.registers[GENERAL_REG1] = Types::$type($num1.$operand(num2));
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.memory.registers[GENERAL_REG1],
                            self.memory.registers[GENERAL_REG2],
                            self.code.data[self.code.ptr],
                        ));
                    }
                };
                ($type: tt, %, $num1: ident) => {
                    if let Types::$type(num2) = self.memory.registers[GENERAL_REG2] {
                        self.memory.registers[GENERAL_REG1] = Types::$type($num1 % num2);
                    } else {
                        return self.panic_rt(ErrTypes::CrossTypeOperation(
                            self.memory.registers[GENERAL_REG1],
                            self.memory.registers[GENERAL_REG2],
                            self.code.data[self.code.ptr],
                        ));
                    }
                };
            }
            use Instructions::*;
            match self.code.data[self.code.ptr] {
                Wr(stack_offset, register) => {
                    let end = self.stack_end();
                    self.memory.stack.data[end - stack_offset] = self.memory.registers[register];
                    self.next_line();
                }
                Rd(stack_offset, reg) => {
                    // print stack offset and end
                    let end = self.stack_end();
                    self.memory.registers[reg] = self.memory.stack.data[end - stack_offset];
                    self.next_line();
                }
                Wrp(value_reg) => {
                    if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                        match kind {
                            PointerTypes::Stack => {
                                self.memory.stack.data[u_size] = self.memory.registers[value_reg];
                            }
                            PointerTypes::Heap(loc) => {
                                self.memory.heap.data[u_size][loc] =
                                    self.memory.registers[value_reg];
                            }
                            PointerTypes::Object => {
                                return self.panic_rt(ErrTypes::Expected(
                                    Types::Pointer(0, PointerTypes::Heap(0)),
                                    self.memory.registers[POINTER_REG],
                                ));
                            }
                            PointerTypes::String => {
                                if let Types::Pointer(dest, PointerTypes::String) =
                                    self.memory.registers[value_reg]
                                {
                                    self.memory.strings.copy_from(u_size, dest)
                                } else {
                                    return self.panic_rt(ErrTypes::Expected(
                                        Types::Pointer(0, PointerTypes::String),
                                        self.memory.registers[value_reg],
                                    ));
                                }
                            }
                            PointerTypes::Char(loc) => {
                                if let Types::Char(chr) = self.memory.registers[value_reg] {
                                    self.memory.strings.pool[u_size][loc] = chr
                                } else {
                                    return self.panic_rt(ErrTypes::Expected(
                                        Types::Char('a'),
                                        self.memory.registers[value_reg],
                                    ));
                                }
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::Pointer(0, PointerTypes::Heap(0)),
                            self.memory.registers[POINTER_REG],
                        ));
                    }
                    self.next_line();
                }
                Rdp(cash_reg) => {
                    if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                        match kind {
                            PointerTypes::Stack => {
                                self.memory.registers[cash_reg] = self.memory.stack.data[u_size];
                            }
                            PointerTypes::Heap(idx) => {
                                self.memory.registers[cash_reg] =
                                    self.memory.heap.data[u_size][idx];
                            }
                            PointerTypes::Object => {
                                return self.panic_rt(ErrTypes::InvalidType(
                                    self.memory.registers[POINTER_REG],
                                    Types::Pointer(0, PointerTypes::Heap(0)),
                                ));
                            }
                            PointerTypes::String => {
                                return self.panic_rt(ErrTypes::InvalidType(
                                    self.memory.registers[POINTER_REG],
                                    Types::Pointer(0, PointerTypes::Heap(0)),
                                ));
                            }
                            PointerTypes::Char(idx) => {
                                self.memory.registers[cash_reg] =
                                    Types::Char(self.memory.strings.pool[u_size][idx]);
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::InvalidType(
                            self.memory.registers[POINTER_REG],
                            Types::Pointer(0, PointerTypes::Heap(0)),
                        ));
                    }
                    self.next_line();
                }
                Rdc(stack_pos, reg) => {
                    self.memory.registers[reg] = self.memory.stack.data[stack_pos];
                    self.next_line();
                }
                Ptr(stack_offset) => {
                    self.memory.registers[GENERAL_REG1] =
                        Types::Pointer(self.stack_end() - stack_offset, PointerTypes::Stack);
                    self.next_line();
                }
                Idx(index_reg) => {
                    if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                        if let Types::Usize(index) = self.memory.registers[index_reg] {
                            match kind {
                                PointerTypes::Object => {
                                    self.memory.registers[POINTER_REG] =
                                        Types::Pointer(u_size, PointerTypes::Heap(index));
                                }
                                PointerTypes::Stack => {
                                    self.memory.registers[POINTER_REG] =
                                        Types::Pointer(u_size + index, PointerTypes::Stack);
                                }
                                PointerTypes::Heap(_) => {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.memory.registers[POINTER_REG],
                                        self.code.data[self.code.ptr],
                                    ));
                                }
                                PointerTypes::Char(_) => {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.memory.registers[POINTER_REG],
                                        self.code.data[self.code.ptr],
                                    ));
                                }
                                PointerTypes::String => {
                                    self.memory.registers[POINTER_REG] =
                                        Types::Pointer(u_size, PointerTypes::Char(index));
                                }
                            }
                        } else {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[POINTER_REG],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[POINTER_REG],
                            self.code.data[self.code.ptr],
                        ));
                    }
                    self.next_line();
                }
                IdxK(index) => {
                    if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                        match kind {
                            PointerTypes::Object => {
                                self.memory.registers[POINTER_REG] =
                                    Types::Pointer(u_size, PointerTypes::Heap(index));
                            }
                            PointerTypes::Stack => {
                                self.memory.registers[POINTER_REG] =
                                    Types::Pointer(u_size + index, PointerTypes::Stack);
                            }
                            PointerTypes::Heap(_) => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[POINTER_REG],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                            PointerTypes::Char(_) => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[POINTER_REG],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                            PointerTypes::String => {
                                self.memory.registers[POINTER_REG] =
                                    Types::Pointer(u_size, PointerTypes::Char(index));
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[POINTER_REG],
                            self.code.data[self.code.ptr],
                        ));
                    }
                    self.next_line();
                }
                Alc(size_reg) => {
                    if let Types::Usize(size) = self.memory.registers[size_reg] {
                        self.memory.registers[POINTER_REG] =
                            Types::Pointer(self.memory.allocate_obj(size), PointerTypes::Object);
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::Usize(0),
                            self.memory.registers[size_reg],
                        ));
                    }
                    self.next_line();
                }
                AlcS(size) => {
                    self.memory.registers[POINTER_REG] =
                        Types::Pointer(self.memory.allocate_obj(size), PointerTypes::Object);
                    self.next_line();
                }
                RAlc(size_reg) => {
                    if let Types::Pointer(u_size, ptr_type) = self.memory.registers[POINTER_REG] {
                        match ptr_type {
                            PointerTypes::Object => {
                                if let Types::Usize(new_size) = self.memory.registers[size_reg] {
                                    self.memory.resize_obj(u_size, new_size);
                                } else {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.memory.registers[size_reg],
                                        self.code.data[self.code.ptr],
                                    ));
                                }
                            }
                            PointerTypes::String => {
                                if let Types::Usize(new_size) = self.memory.registers[size_reg] {
                                    self.memory.strings.pool[u_size].resize(new_size, 0 as char);
                                } else {
                                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                                        self.memory.registers[size_reg],
                                        self.code.data[self.code.ptr],
                                    ));
                                }
                            }
                            _ => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[POINTER_REG],
                                    self.code.data[self.code.ptr],
                                ))
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[POINTER_REG],
                            self.code.data[self.code.ptr],
                        ));
                    }
                    self.next_line();
                }
                Dalc => {
                    if let Types::Pointer(u_size, ptr_type) = self.memory.registers[POINTER_REG] {
                        match ptr_type {
                            PointerTypes::Object => {
                                self.memory.deallocate_obj(u_size);
                            }
                            PointerTypes::String => {
                                self.memory.deallocate_string(u_size);
                            }
                            _ => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[POINTER_REG],
                                    self.code.data[self.code.ptr],
                                ))
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[POINTER_REG],
                            self.code.data[self.code.ptr],
                        ));
                    }
                    self.next_line();
                }
                Sweep => {
                    self.memory.sweep();
                    self.next_line();
                }
                SweepUnoptimized => {
                    self.memory.sweep_unoptimized();
                    self.next_line();
                }
                Goto(pos) => {
                    self.code.ptr = pos;
                }
                Jump(pos) => {
                    self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr = self.code.ptr;
                    self.code.ptr = pos;
                }
                Gotop => {
                    if let Types::CodePointer(u_size) = self.memory.registers[CODE_PTR_REG] {
                        self.code.ptr = u_size
                    } else {
                        return self.panic_rt(ErrTypes::InvalidType(
                            self.memory.registers[CODE_PTR_REG],
                            Types::CodePointer(0),
                        ));
                    }
                }
                Brnc(pos1, pos2) => {
                    if let Types::Bool(bool) = self.memory.registers[GENERAL_REG1] {
                        self.code.ptr = if bool { pos1 } else { pos2 };
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[GENERAL_REG1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                Ret => {
                    self.code.ptr = self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr;
                    self.memory.stack.ptr -= 1;
                    self.next_line();
                }
                Back => {
                    self.code.ptr = self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr;
                    self.next_line();
                }
                Ufrz => {
                    for i in 0..FREEZED_REG_SIZE {
                        self.memory.registers[i] =
                            self.memory.stack.call_stack[self.memory.stack.ptr].reg_freeze[i]
                    }
                    self.next_line();
                }
                Res(size, pointers_len) => {
                    let end = self.stack_end() + size;
                    self.memory.stack.ptr += 1;
                    if self.memory.stack.ptr >= self.memory.stack.call_stack.len() {
                        if self.memory.stack.ptr > self.memory.stack.call_stack.len() {
                            loop {
                                println!("Samik mel pravdu, ale tohle stejne nikdy neuvidis ;p");
                            }
                        }
                        return self.panic_rt(ErrTypes::StackOverflow);
                    }
                    self.memory.stack.call_stack[self.memory.stack.ptr].end = end;
                    self.memory.stack.call_stack[self.memory.stack.ptr].pointers_len = pointers_len;
                    if end > self.memory.stack.data.len() {
                        self.memory.stack.data.resize(end, Types::Null);
                    }
                    self.next_line();
                }
                Frz => {
                    self.memory.stack.call_stack[self.memory.stack.ptr]
                        .reg_freeze
                        .clone_from_slice(&self.memory.registers[..3]);
                    self.next_line();
                }
                Swap(reg1, reg2) => {
                    let temp = self.memory.registers[reg1];
                    self.memory.registers[reg1] = self.memory.registers[reg2];
                    self.memory.registers[reg2] = temp;
                    self.next_line();
                }
                Move(reg1, reg2) => {
                    self.memory.registers[reg2] = self.memory.registers[reg1];
                    self.next_line();
                }
                Add => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, add, num1),
                        Types::Float(num1) => operation!(Float, add, num1),
                        Types::Byte(num1) => operation!(Byte, add, num1),
                        Types::Usize(num1) => operation!(Usize, add, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Sub => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, sub, num1),
                        Types::Float(num1) => operation!(Float, sub, num1),
                        Types::Byte(num1) => operation!(Byte, sub, num1),
                        Types::Usize(num1) => operation!(Usize, sub, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Mul => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, mul, num1),
                        Types::Float(num1) => operation!(Float, mul, num1),
                        Types::Byte(num1) => operation!(Byte, mul, num1),
                        Types::Usize(num1) => operation!(Usize, mul, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Div => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, div, num1),
                        Types::Float(num1) => operation!(Float, div, num1),
                        Types::Byte(num1) => operation!(Byte, div, num1),
                        Types::Usize(num1) => operation!(Usize, div, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Mod => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, %, num1),
                        Types::Float(num1) => operation!(Float, %, num1),
                        Types::Byte(num1) => operation!(Byte, %, num1),
                        Types::Usize(num1) => operation!(Usize, %, num1),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Equ => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, eq, num1, bool),
                        Types::Float(num1) => operation!(Float, eq, num1, bool),
                        Types::Byte(num1) => operation!(Byte, eq, num1, bool),
                        Types::Usize(num1) => operation!(Usize, eq, num1, bool),
                        Types::Pointer(num1, _) => operation!(ptr, eq, num1, bool),
                        Types::Bool(var1) => operation!(Bool, eq, var1, bool),
                        Types::Char(char1) => operation!(Char, eq, char1, bool),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Grt => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, gt, num1, bool),
                        Types::Float(num1) => operation!(Float, gt, num1, bool),
                        Types::Byte(num1) => operation!(Byte, gt, num1, bool),
                        Types::Usize(num1) => operation!(Usize, gt, num1, bool),
                        Types::Char(char1) => operation!(Char, gt, char1, bool),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Less => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Int(num1) => operation!(Int, lt, num1, bool),
                        Types::Float(num1) => operation!(Float, lt, num1, bool),
                        Types::Byte(num1) => operation!(Byte, lt, num1, bool),
                        Types::Usize(num1) => operation!(Usize, lt, num1, bool),
                        Types::Char(char1) => operation!(Char, lt, char1, bool),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                And => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Bool(var1) => {
                            if let Types::Bool(var2) = self.memory.registers[GENERAL_REG2] {
                                self.memory.registers[GENERAL_REG1] = Types::Bool(var1 && var2)
                            } else {
                                return self.panic_rt(ErrTypes::CrossTypeOperation(
                                    self.memory.registers[GENERAL_REG1],
                                    self.memory.registers[GENERAL_REG2],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Or => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Bool(var1) => {
                            if let Types::Bool(var2) = self.memory.registers[GENERAL_REG2] {
                                self.memory.registers[GENERAL_REG1] = Types::Bool(var1 || var2)
                            } else {
                                return self.panic_rt(ErrTypes::CrossTypeOperation(
                                    self.memory.registers[GENERAL_REG1],
                                    self.memory.registers[GENERAL_REG2],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Not => {
                    match self.memory.registers[GENERAL_REG1] {
                        Types::Bool(var) => self.memory.registers[GENERAL_REG1] = Types::Bool(!var),
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[GENERAL_REG1],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Cal(lib, fun_id) => {
                    match self.libs[lib].call(
                        fun_id,
                        PublicData {
                            memory: &mut self.memory,
                            code: &mut self.code,
                        },
                    ) {
                        Ok(value) => {
                            if let Types::Void = value {
                            } else {
                                self.memory.registers[RETURN_REG] = value;
                            }
                        }
                        Err(err) => {
                            return self.panic_rt(err);
                        }
                    }
                    self.next_line();
                }
                Mtd(obj, trt, method) => {
                    if let Types::NonPrimitive(id) = self.memory.registers[obj] {
                        if let Some(method) =
                            self.memory.non_primitives[id].methods[trt].get(method)
                        {
                            self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr =
                                self.code.ptr;
                            self.code.ptr = *method;
                        } else {
                            return self.panic_rt(ErrTypes::MethodNotFound);
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[obj],
                            self.code.data[self.code.ptr],
                        ));
                    }
                    /*self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr = self.code.ptr;
                    self.code.ptr = self.memory.non_primitives[obj].methods[trt][method];*/
                }
                End => {
                    return false;
                }
                Debug(reg) => {
                    println!("{:+}", self.memory.registers[reg]);
                    self.next_line();
                }
                Len(reg) => {
                    if let Types::NonPrimitive(kind) = self.memory.registers[reg] {
                        self.memory.registers[reg] =
                            Types::Usize(self.memory.non_primitives[kind].len)
                    } else if let Types::Pointer(u_size, kind) = self.memory.registers[reg] {
                        match kind {
                            PointerTypes::Object => {
                                self.memory.registers[reg] =
                                    Types::Usize(self.memory.heap.data[u_size].len())
                            }
                            PointerTypes::String => {
                                self.memory.registers[reg] =
                                    Types::Usize(self.memory.strings.pool[u_size].len())
                            }
                            _ => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[reg],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::InvalidType(
                            self.memory.registers[reg],
                            Types::NonPrimitive(0),
                        ));
                    }
                    self.next_line()
                }
                CpRng(original, new, len) => {
                    let new_ptr = if let Types::Pointer(u_size, kind) = self.memory.registers[new] {
                        (u_size, kind)
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[new],
                            CpRng(0, 0, 0),
                        ));
                    };
                    if let Types::Pointer(u_size, kind) = self.memory.registers[original] {
                        for i in 0..len {
                            let value = match kind {
                                PointerTypes::Object => self.memory.heap.data[u_size][i],
                                PointerTypes::String => {
                                    Types::Char(self.memory.strings.pool[u_size][i])
                                }
                                PointerTypes::Stack => self.memory.stack.data[u_size + i],
                                PointerTypes::Heap(idx) => self.memory.heap.data[u_size][i + idx],
                                PointerTypes::Char(idx) => {
                                    Types::Char(self.memory.strings.pool[u_size][i + idx])
                                }
                            };
                            match new_ptr.1 {
                                PointerTypes::Object => {
                                    self.memory.heap.data[new_ptr.0][i] = value;
                                }
                                PointerTypes::String => {
                                    self.memory.strings.pool[new_ptr.0][i] = value.get_char();
                                }
                                PointerTypes::Stack => {
                                    self.memory.stack.data[new_ptr.0 + i] = value;
                                }
                                PointerTypes::Heap(idx) => {
                                    self.memory.heap.data[new_ptr.0][idx + i] = value;
                                }
                                PointerTypes::Char(idx) => {
                                    self.memory.strings.pool[new_ptr.0][idx + i] = value.get_char();
                                }
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[original],
                            CpRng(0, 0, 0),
                        ));
                    }
                }
                TRng(val, len) => {
                    let value = self.memory.registers[val];
                    if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                        for i in 0..len {
                            match kind {
                                PointerTypes::Object => {
                                    self.memory.heap.data[u_size][i] = value;
                                }
                                PointerTypes::Stack => {
                                    self.memory.stack.data[u_size + i] = value;
                                }
                                PointerTypes::Heap(idx) => {
                                    self.memory.heap.data[u_size][i + idx] = value;
                                }
                                PointerTypes::Char(idx) => {
                                    self.memory.strings.pool[u_size][i + idx] = value.get_char();
                                }
                                PointerTypes::String => {
                                    self.memory.strings.pool[u_size][i] = value.get_char();
                                }
                            }
                        }
                    }
                }
                Type(reg1, reg2) => {
                    use std::mem::discriminant;
                    self.memory.registers[reg2] = Types::Bool(
                        discriminant(&self.memory.registers[reg1])
                            == discriminant(&self.memory.registers[reg2]),
                    );
                    self.next_line();
                }
                NPType(np_reg, id) => {
                    if let Types::NonPrimitive(id_dyn) = self.memory.registers[np_reg] {
                        self.memory.registers[GENERAL_REG3] = Types::Bool(id == id_dyn);
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::NonPrimitive(0),
                            self.memory.registers[np_reg],
                        ));
                    }
                }
                Cast(reg1, ttype) => {
                    if let Err(err) = Self::cast(&mut self.memory.registers, reg1, ttype) {
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
                        code_ptr: self.code.ptr,
                        id: None,
                        cs_ptr: self.memory.stack.ptr,
                    }) {
                        return self.panic_rt(err);
                    }
                    self.next_line()
                }
                CatchId(id) => {
                    if let Err(err) = self.catches.push(runtime_types::Catch {
                        code_ptr: self.code.ptr,
                        id: Some(id),
                        cs_ptr: self.memory.stack.ptr,
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
                    if let Types::Pointer(u_size, PointerTypes::String) = self.memory.registers[reg]
                    {
                        self.memory.registers[POINTER_REG] =
                            Types::Pointer(self.memory.strings.copy(u_size), PointerTypes::String);
                    } else {
                        return self.panic_rt(ErrTypes::Expected(
                            Types::Pointer(0, PointerTypes::String),
                            self.memory.registers[reg],
                        ));
                    }
                    self.next_line();
                }
                StrNew => {
                    self.memory.registers[POINTER_REG] =
                        Types::Pointer(self.memory.strings.new(), PointerTypes::String);
                    self.next_line();
                }
                IntoStr(reg) => {
                    match self.memory.registers[reg] {
                        Types::Bool(b) => {
                            self.memory.registers[POINTER_REG] = Types::Pointer(
                                self.memory.strings.from(b.to_string().chars().collect()),
                                PointerTypes::String,
                            );
                        }
                        Types::Char(c) => {
                            self.memory.registers[POINTER_REG] = Types::Pointer(
                                self.memory.strings.from(vec![c]),
                                PointerTypes::String,
                            );
                        }
                        Types::Int(i) => {
                            self.memory.registers[POINTER_REG] = Types::Pointer(
                                self.memory.strings.from(i.to_string().chars().collect()),
                                PointerTypes::String,
                            );
                        }
                        Types::Float(f) => {
                            self.memory.registers[POINTER_REG] = Types::Pointer(
                                self.memory.strings.from(f.to_string().chars().collect()),
                                PointerTypes::String,
                            );
                        }
                        Types::Null => {
                            self.memory.registers[POINTER_REG] = Types::Pointer(
                                self.memory.strings.from("null".chars().collect()),
                                PointerTypes::String,
                            );
                        }
                        Types::NonPrimitive(kind) => {
                            self.memory.registers[POINTER_REG] = Types::Pointer(
                                self.memory.strings.from(kind.to_string().chars().collect()),
                                PointerTypes::String,
                            );
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::Expected(
                                Types::Pointer(0, PointerTypes::String),
                                self.memory.registers[reg],
                            ));
                        }
                    }
                    self.next_line();
                }
                StdOut(reg) => {
                    match self.memory.registers[reg] {
                        Types::Pointer(u_size, kind) => match kind {
                            PointerTypes::String => {
                                let mut temp = String::new();
                                for chr in &self.memory.strings.pool[u_size] {
                                    temp.push(*chr);
                                }
                                print!("{temp}");
                            }
                            _ => {
                                unreachable!("StdOut: Invalid pointer type");
                            }
                        },
                        _ => {
                            unreachable!("StdOut: Invalid type");
                        }
                    }
                    self.next_line()
                }
                Panic => {
                    self.enter_panic();
                    self.next_line();
                }
            }
            return true;
        }
        fn stack_end(&self) -> usize {
            self.memory.stack.call_stack[self.memory.stack.ptr].end
        }
        fn next_line(&mut self) {
            self.code.ptr += 1;
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
        fn panic_rt(&mut self, kind: ErrTypes) -> bool {
            if self.enter_panic() {
                return true;
            }
            self.break_code = Some(self.code.ptr);
            println!("{}", get_message(&kind, Some((self.code.ptr, 0))));
            self.exit_code = ExitCodes::Internal(kind);
            false
        }
        /// This function is called when an exception is thrown. It will search for a catch block
        /// that matches the exception type. If it finds one, it will set the code pointer to the
        /// catch block and set the stack pointer to the catch block's stack pointer. If it doesn't
        /// find one, it will set the exit code to exception and return false. (indicating that the VM should exit)
        fn enter_panic(&mut self) -> bool {
            let mut i = self.catches.catches_ptr;
            loop {
                if i == 0 {
                    self.exit_code = ExitCodes::Exception;
                    return false;
                }
                i -= 1;
                if let Some(n) = self.catches.cache[i].id {
                    if let Types::NonPrimitive(e_type) = self.memory.registers[RETURN_REG] {
                        if n == e_type {
                            self.code.ptr = self.catches.cache[i].code_ptr;
                            self.memory.stack.ptr = self.catches.cache[i].cs_ptr;
                        }
                    }
                    break;
                } else {
                    self.code.ptr = self.catches.cache[i].code_ptr;
                    self.memory.stack.ptr = self.catches.cache[i].cs_ptr;
                    break;
                }
            }
            self.catches.truncate(i);
            true
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
            pub memory: Memory,
            pub code: Code,
            pub break_code: Option<usize>,
            pub catches: Catches,
            pub exit_code: ExitCodes,
            pub libs: Vec<Box<dyn Library>>,
        }
        pub struct Memory {
            pub stack: Stack,
            pub registers: Registers,
            pub heap: Heap,
            pub strings: Strings,
            pub non_primitives: Vec<NonPrimitiveType>,
        }
        impl Memory {
            // allocator starts here
            pub fn allocate_obj(&mut self, size: usize) -> usize {
                let mut data = Vec::new();
                data.resize(size, Types::Null);
                if let Some(idx) = self.heap.garbage.pop() {
                    self.heap.data[idx] = data;
                    return idx;
                }
                self.heap.data.push(data);
                self.heap.data.len() - 1
            }
            pub fn deallocate_obj(&mut self, idx: usize) -> bool {
                if idx >= self.heap.data.len() {
                    return false;
                }
                if idx == self.heap.data.len() - 1 {
                    self.heap.data.pop();
                    // get largest index of non garbage obj using last obj and truncate
                    let last = self.last_obj();
                    self.heap.data.truncate(last);
                    return true;
                }
                self.heap.garbage.push(idx);
                self.heap.data[idx].clear();
                true
            }
            pub fn last_string(&mut self) -> usize {
                if self.strings.pool.is_empty() {
                    return 0;
                }
                // find first string that is garbage and following strings are garbage and dont remove any strings from garbage
                let mut i = self.strings.pool.len() - 1;
                loop {
                    // if string is garbage and all strings after it are garbage then return i + 1
                    if self.strings.garbage.iter().any(|e| *e == i) {
                        if i == 0 {
                            return 0;
                        }
                        i -= 1;
                    } else {
                        return i + 1;
                    }
                }
            }
            pub fn last_obj(&mut self) -> usize {
                if self.heap.data.is_empty() {
                    return 0;
                }
                // find first object that is garbage and following objects are garbage and dont remove any objects from garbage
                let mut i = self.heap.data.len() - 1;
                loop {
                    // if object is garbage and all objects after it are garbage then return i + 1
                    if self.heap.garbage.iter().any(|e| *e == i) {
                        if i == 0 {
                            return 0;
                        }
                        i -= 1;
                    } else {
                        return i + 1;
                    }
                }
            }
            pub fn deallocate_string(&mut self, idx: usize) -> bool {
                if idx >= self.strings.pool.len() {
                    return false;
                }
                if idx == self.strings.pool.len() - 1 {
                    self.strings.pool.pop();
                    // get largest index of non garbage string and truncate
                    let last = self.last_string();
                    self.strings.pool.truncate(last);
                    return true;
                }
                self.strings.garbage.push(idx);
                self.strings.pool[idx].clear();
                true
            }
            pub fn resize_obj(&mut self, heap_idx: usize, new_size: usize) {
                self.heap.data[heap_idx].resize(new_size, Types::Null)
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
            pub fn sweep_marked(&mut self, marked: (Vec<bool>, Vec<bool>)) {
                self.sweep_marked_obj(marked.0);
                self.sweep_marked_string(marked.1);
                let last = self.last_string();
                self.strings.pool.truncate(last);
                let last = self.last_obj();
                self.heap.data.truncate(last);
            }
            pub fn sweep_marked_obj(&mut self, marked: Vec<bool>) {
                if let Some(idx) = marked.iter().rposition(|x| !*x) {
                    self.heap.data.truncate(idx + 1);
                } else {
                    self.heap.data.clear();
                    return;
                }
                for (i, mark) in marked.iter().enumerate() {
                    if i == self.heap.data.len() {
                        return;
                    }
                    if *mark {
                        self.heap.data[i].clear();
                        if !self.heap.garbage.contains(&i) {
                            self.heap.garbage.push(i);
                        }
                    }
                }
            }
            pub fn sweep_marked_string(&mut self, marked: Vec<bool>) {
                // find first string that is garbage and following strings are garbage and then remove them from garbage
                if let Some(idx) = marked.iter().rposition(|x| !*x) {
                    self.strings.pool.truncate(idx + 1);
                } else {
                    self.strings.pool.clear();
                    return;
                }
                // remove all strings that are marked
                for (i, mark) in marked.iter().enumerate() {
                    if i == self.strings.pool.len() {
                        continue;
                    }
                    if *mark {
                        self.strings.pool[i].clear();
                        if !self.strings.garbage.contains(&i) {
                            self.strings.garbage.push(i);
                        }
                    }
                }
            }
            pub fn mark_unoptimized(&mut self) -> (Vec<bool>, Vec<bool>) {
                let mut marked_obj = Vec::new();
                let mut marked_str = Vec::new();
                marked_obj.resize(self.heap.data.len(), true);
                marked_str.resize(self.strings.pool.len(), true);
                self.mark_registers(&mut marked_obj, &mut marked_str);
                self.mark_range((0, self.stack.data.len()), &mut marked_obj, &mut marked_str);
                (marked_obj, marked_str)
            }
            pub fn mark(&mut self) -> (Vec<bool>, Vec<bool>) {
                let mut call_stack_idx = 1;
                let mut marked = Vec::new();
                let mut marked_str = Vec::new();
                marked.resize(self.heap.data.len(), true);
                marked_str.resize(self.strings.pool.len(), true);
                self.mark_registers(&mut marked, &mut marked_str);
                while call_stack_idx <= self.stack.ptr {
                    let cs = self.stack.call_stack[call_stack_idx];
                    let prev_cs = self.stack.call_stack[call_stack_idx - 1];
                    self.mark_range(
                        (prev_cs.end, prev_cs.end + cs.pointers_len),
                        &mut marked,
                        &mut marked_str,
                    );
                    call_stack_idx += 1;
                }
                (marked, marked_str)
            }
            pub fn mark_obj(&mut self, obj_idx: usize, marked: &mut Vec<bool>) {
                if !marked[obj_idx] {
                    return;
                }
                marked[obj_idx] = false;
                for idx in 0..self.heap.data[obj_idx].len() {
                    let member = self.heap.data[obj_idx][idx];
                    if let Types::Pointer(u_size, PointerTypes::Object) = member {
                        self.mark_obj(u_size, marked);
                    } else if let Types::Pointer(u_size, PointerTypes::Heap(_)) = member {
                        self.mark_obj(u_size, marked);
                    }
                }
            }
            pub fn mark_string(&mut self, str_idx: usize, marked: &mut Vec<bool>) {
                marked[str_idx] = false;
            }
            pub fn mark_range(
                &mut self,
                range: (usize, usize),
                marked_obj: &mut Vec<bool>,
                marked_string: &mut Vec<bool>,
            ) {
                for idx in range.0..range.1 {
                    if let Types::Pointer(u_size, PointerTypes::Heap(_)) = self.stack.data[idx] {
                        self.mark_obj(u_size, marked_obj);
                    } else if let Types::Pointer(u_size, PointerTypes::Object) =
                        self.stack.data[idx]
                    {
                        self.mark_obj(u_size, marked_obj);
                    } else if let Types::Pointer(u_size, PointerTypes::String) =
                        self.stack.data[idx]
                    {
                        self.mark_string(u_size, marked_string);
                    }
                }
            }
            pub fn mark_registers(&mut self, marked: &mut Vec<bool>, marked_str: &mut Vec<bool>) {
                for reg in self.registers {
                    if let Types::Pointer(u_size, PointerTypes::Heap(_)) = reg {
                        self.mark_obj(u_size, marked);
                    } else if let Types::Pointer(u_size, PointerTypes::Object) = reg {
                        self.mark_obj(u_size, marked);
                    } else if let Types::Pointer(u_size, PointerTypes::String) = reg {
                        self.mark_string(u_size, marked_str);
                    }
                }
            }
        }
        pub struct Stack {
            pub data: Vec<Types>,
            pub ptr: usize,
            pub call_stack: [CallStack; CALL_STACK_SIZE],
        }
        pub struct Heap {
            pub data: Vec<Vec<Types>>,
            pub garbage: Vec<usize>,
        }
        pub struct Strings {
            pub pool: Vec<Vec<char>>,
            pub garbage: Vec<usize>,
        }
        impl Strings {
            /// Creates a new empty string and returns the location of the string
            pub fn new(&mut self) -> usize {
                // either push a new string or occupy a deleted string
                if let Some(loc) = self.garbage.pop() {
                    self.pool[loc] = Vec::new();
                    loc
                } else {
                    self.pool.push(Vec::new());
                    self.pool.len() - 1
                }
            }
            pub fn from_String(&mut self, str: String) -> usize {
                // either push a new string or occupy a deleted string
                if let Some(loc) = self.garbage.pop() {
                    self.pool[loc] = str.chars().collect();
                    loc
                } else {
                    self.pool.push(str.chars().collect());
                    self.pool.len() - 1
                }
            }
            ///  Creates a new copied string and returns the location of the string
            pub fn from(&mut self, str: Vec<char>) -> usize {
                // either push a new string or occupy a deleted string
                if let Some(loc) = self.garbage.pop() {
                    self.pool[loc] = str;
                    loc
                } else {
                    self.pool.push(str);
                    self.pool.len() - 1
                }
            }
            /// Copies a string from one location to a new location and returns the new location
            pub fn copy(&mut self, loc: usize) -> usize {
                // either push a new string or occupy a deleted string
                if let Some(new_loc) = self.garbage.pop() {
                    self.pool[new_loc] = self.pool[loc].clone();
                    new_loc
                } else {
                    self.pool.push(self.pool[loc].clone());
                    self.pool.len() - 1
                }
            }
            /// Copies a string from one location to another location.
            pub fn copy_from(&mut self, orig: usize, dest: usize) {
                self.pool[dest] = self.pool[orig].clone()
            }
            pub fn concat(&mut self, left: usize, right: usize) -> usize {
                let mut temp = self.pool[left].clone();
                temp.extend(self.pool[right].iter());
                self.from(temp)
            }
            pub fn push_string_array(&mut self, arr: Vec<&str>) -> Vec<usize> {
                let mut temp = Vec::with_capacity(arr.len());
                for str in arr {
                    temp.push(self.from_String(str.to_owned()));
                }
                temp
            }
            pub fn to_string(&self, loc: usize) -> String {
                self.pool[loc].iter().collect()
            }
        }
        pub struct Code {
            pub data: Vec<Instructions>,
            pub ptr: usize,
        }
        pub struct PublicData<'a> {
            pub memory: &'a mut Memory,
            pub code: &'a mut Code,
        }
        #[derive(Debug, Clone)]
        pub struct Garbage {
            pub heap: Vec<usize>,
            pub string_pool: Vec<usize>,
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
        #[derive(Debug, Clone)]
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
            // null represents an empty value
            Null,
            // void represents a value that is not meant to be used
            Void,
            /// header for non-primitive types
            /// ID
            NonPrimitive(usize),
        }
        impl Types {
            /// may panic, so use this only if you are 100% certain that you got a character
            pub fn get_char(&self) -> char {
                if let Types::Char(chr) = self {
                    return *chr;
                }
                unreachable!()
            }
        }
        #[derive(Clone, Copy, Debug)]
        pub enum NonPrimitiveTypes {
            Array,
            Struct,
        }
        #[derive(Debug, Clone)]
        pub struct NonPrimitiveType {
            pub name: String,
            pub kind: NonPrimitiveTypes,
            pub len: usize,
            pub pointers: usize,
            // first index is trait id, second is method id
            pub methods: Vec<Vec<usize>>,
        }
        use std::{clone, fmt, rc::Rc, sync::Arc};

        use super::{
            runtime_error::{self, ErrTypes},
            Library,
        };
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
                        Types::Void => write!(f, "Void"),
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
                        Types::Void => write!(f, "Void"),
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
                        Types::Void => write!(f, "Void"),
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
            /// location in string pool
            /// never expires
            String,
            /// char
            ///
            /// location and index in string pool
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
            /// Free: | frees heap(<reg>)
            Dalc,
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
            /// Length: reg | sets reg to Usize(size of an object)
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
            /// Method: struct trait method | takes struct and calls method on it, assuming it implements trait  
            Mtd(usize, usize, usize),
            /// Panic | program enters panic mode, returning from all stacks until exception is caught
            Panic,
            /// Catch | catches an error and returns program to normal mode, cached if read in normal mode
            Catch,
            /// Catch ID: id | same as normal catch but responds only to exception with same id
            CatchId(usize),
            /// Delete catch | deletes one catch instruction from cache
            DelCatch,
            /// Non-primitive type: np_reg ID | compares reg(np_reg).id assuming it belongs to Non-primitive type with ID
            NPType(usize, usize),
            /// String new | creates new string and stores pointer in reg(POINTER_REGISTER)
            StrNew,
            /// String copy: str_reg | copies string from reg(str_reg) to new string and stores pointer in reg(POINTER_REGISTER)
            StrCpy(usize),
            /// Into string: val_reg | converts value on reg(value_reg) to string and stores pointer in reg(POINTER_REG)
            IntoStr(usize),
            /// Standard out: val_reg | outputs value on reg(value_reg) to stdout
            StdOut(usize),
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
                    Instructions::Mtd(_, _, _) => "Method",
                    Instructions::Break(_) => "Break",
                    Instructions::Panic => "Panic",
                    Instructions::Catch => "Catch",
                    Instructions::CatchId(_) => "Catch",
                    Instructions::DelCatch => "DeleteCatch",
                    Instructions::NPType(_, _) => "NonPrimitiveType",
                    Instructions::StrNew => "StringNew",
                    Instructions::StrCpy(_) => "StringCopy",
                    Instructions::StdOut(_) => "StandardOutput",
                    Instructions::Dalc => "Deallocate",
                    Instructions::IntoStr(_) => "IntoString",
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
    pub mod runtime_error {
        use super::runtime_types::*;
        #[derive(Debug, Clone)]
        pub enum ErrTypes {
            CrossTypeOperation(Types, Types, Instructions),
            WrongTypeOperation(Types, Instructions),
            InvalidType(Types, Types),
            Expected(Types, Types),
            ImplicitCast(Types, Types),
            StackOverflow,
            CatchOwerflow,
            MethodNotFound,
            Message(String),
        }
        fn gen_message(header: String, line: Option<(usize, usize)>, err_no: u8) -> String {
            return if let Some(line) = line {
                //                    code                      header                      line     column
                format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mAt: line {}, column {}.\x1b[0m", line.0, line.1)
            } else {
                format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mLocation unspecified.\x1b[0m")
            };
        }
        pub fn get_message(kind: &ErrTypes, line: Option<(usize, usize)>) -> String {
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
                    format!(
                        "Cast error: Can not implicitly cast type {type1:#} into type {type2:#}"
                    ),
                    4,
                ),
                ErrTypes::StackOverflow => (format!("Stack overflow"), 5), // TODO: impl this
                ErrTypes::CatchOwerflow => (format!("Catch overflow"), 6),
                ErrTypes::MethodNotFound => (format!("Method not found"), 7),
                ErrTypes::Message(msg) => (msg.clone(), 8),
            };
            let message = gen_message(data.0, line, data.1);
            message
        }
    }
    /// public interface for the library to be used by the interpreter and the compiler
    pub trait Library {
        /// calls a function from the library with the given id and arguments and returns the result
        /// mem: (stack, heap, string pool)
        fn call(&mut self, id: usize, mem: PublicData) -> Result<Types, ErrTypes>;
        /// returns the name of the library
        fn name(&self) -> String;
        /// returns the functions of the library
        /// (name, id)
        /// name must be in the format of
        /// name: fun name<T>(args: T): T
        ///
        /// this is only enforced by the compiler
        /// and not by the interpreter
        fn register(&self) -> Vec<(String, usize)>;
    }
}
