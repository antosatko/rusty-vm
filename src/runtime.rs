pub mod runtime {
    use std::ops::Add;
    use std::ops::Div;
    use std::ops::Mul;
    use std::ops::Sub;
    use std::vec;

    use colored::Colorize;

    use super::runtime_error::*;
    use super::runtime_types::*;

    impl Context {
        pub fn new() -> Self {
            Self {
                stack: vec![],
                call_stack: [CallStack {
                    end: 0,
                    code_ptr: 0,
                    reg_freeze: [Types::Null; 3],
                    pointers_len: 0,
                }; 255],
                registers: [Types::Null; REGISTER_SIZE],
                code: vec![],
                code_ptr: 0,
                heap: vec![],
                garbage: vec![],
                stack_ptr: 0,
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
                        return panic_rt(ErrTypes::CrossTypeOperation(
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
                        return panic_rt(ErrTypes::CrossTypeOperation(
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
                        return panic_rt(ErrTypes::CrossTypeOperation(
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
                        return panic_rt(ErrTypes::CrossTypeOperation(
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
                                return panic_rt(ErrTypes::Expected(
                                    Types::Pointer(0, PointerTypes::Heap(0)),
                                    self.registers[POINTER_REG],
                                ));
                            }
                        }
                    } else {
                        return panic_rt(ErrTypes::Expected(
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
                                return panic_rt(ErrTypes::InvalidType(
                                    self.registers[POINTER_REG],
                                    Types::Pointer(0, PointerTypes::Heap(0)),
                                ));
                            }
                        }
                    } else {
                        return panic_rt(ErrTypes::InvalidType(
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
                                    return panic_rt(ErrTypes::WrongTypeOperation(
                                        self.registers[POINTER_REG],
                                        self.code[self.code_ptr],
                                    ));
                                }
                            }
                        } else {
                            return panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[POINTER_REG],
                                self.code[self.code_ptr],
                            ));
                        }
                    } else {
                        return panic_rt(ErrTypes::WrongTypeOperation(
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
                                return panic_rt(ErrTypes::WrongTypeOperation(
                                    self.registers[POINTER_REG],
                                    self.code[self.code_ptr],
                                ));
                            }
                        }
                    } else {
                        return panic_rt(ErrTypes::WrongTypeOperation(
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
                        return panic_rt(ErrTypes::Expected(
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
                                    return panic_rt(ErrTypes::WrongTypeOperation(
                                        self.registers[size_reg],
                                        self.code[self.code_ptr],
                                    ));
                                }
                            }
                            _ => {
                                return panic_rt(ErrTypes::WrongTypeOperation(
                                    self.registers[POINTER_REG],
                                    self.code[self.code_ptr],
                                ))
                            }
                        }
                    } else {
                        return panic_rt(ErrTypes::WrongTypeOperation(
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
                        return panic_rt(ErrTypes::InvalidType(
                            self.registers[CODE_PTR_REG],
                            Types::CodePointer(0),
                        ));
                    }
                }
                Brnc(pos1, pos2) => {
                    if let Types::Bool(bool) = self.registers[GENERAL_REG1] {
                        self.code_ptr = if bool { pos1 } else { pos2 };
                    } else {
                        return panic_rt(ErrTypes::WrongTypeOperation(
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
                        return panic_rt(ErrTypes::StackOverflow);
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
                        .copy_from_slice(&self.registers[..3]);
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                                return panic_rt(ErrTypes::CrossTypeOperation(
                                    self.registers[GENERAL_REG1],
                                    self.registers[GENERAL_REG2],
                                    self.code[self.code_ptr],
                                ));
                            }
                        }
                        _ => {
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                                return panic_rt(ErrTypes::CrossTypeOperation(
                                    self.registers[GENERAL_REG1],
                                    self.registers[GENERAL_REG2],
                                    self.code[self.code_ptr],
                                ));
                            }
                        }
                        _ => {
                            return panic_rt(ErrTypes::WrongTypeOperation(
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
                            return panic_rt(ErrTypes::WrongTypeOperation(
                                self.registers[GENERAL_REG1],
                                self.code[self.code_ptr],
                            ));
                        }
                    }
                    self.next_line();
                }
                Cal(_procedure, _args) => {}
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
                                return panic_rt(ErrTypes::PointerInBrokenState);
                            }
                        } else {
                            return panic_rt(ErrTypes::NotObject(self.registers[POINTER_REG]));
                        }
                    } else {
                        return panic_rt(ErrTypes::WrongTypeOperation(
                            self.registers[POINTER_REG],
                            self.code[self.code_ptr],
                        ));
                    } 
                    self.next_line();*/
                    todo!();
                }
                Type(reg1, reg2) => {
                    use std::mem::discriminant;
                    self.registers[reg2] = Types::Bool(
                        discriminant(&self.registers[reg1]) == discriminant(&self.registers[reg2]),
                    );
                    self.next_line();
                }
                Cast(reg1, ttype) => {
                    if !Self::cast(&mut self.registers, reg1, ttype) {
                        return false;
                    }
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
        fn cast(registers: &mut Registers, reg1: usize, reg2: usize) -> bool {
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
                    _ => return panic_rt(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
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
                    _ => return panic_rt(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
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
                    _ => return panic_rt(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
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
                    _ => return panic_rt(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
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
                    _ => return panic_rt(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
                },
                _ => return panic_rt(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
            }
            true
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
    }
}

pub mod runtime_error {
    use super::runtime_types::*;
    #[repr(C, u8)]
    pub enum ErrTypes {
        CrossTypeOperation(Types, Types, Instructions),
        WrongTypeOperation(Types, Instructions),
        InvalidType(Types, Types),
        Expected(Types, Types),
        ImplicitCast(Types, Types),
        StackOverflow,
    }
    fn gen_message(header: String, line: Option<(usize, usize)>, err_no: u8) -> String {
        return if let Some(line) = line {
            //                    code                      header                      line     column
            format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mAt: line {}, column {}.\x1b[0m", line.0, line.1)
        } else {
            format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mLocation unspecified.\x1b[0m")
        };
    }
    pub fn panic_rt(kind: ErrTypes) -> bool {
        let data: String = match kind {
            ErrTypes::CrossTypeOperation(var1, var2, instr) => {
                format!("Operation '{instr}' failed: Cross-type operation {var1:+}, {var2:+}")
            }
            ErrTypes::WrongTypeOperation(var1, instr) => {
                format!("Operation '{instr}' failed: Wrong-type operation {var1:+}")
            }
            ErrTypes::InvalidType(typ, operation) => {
                format!("Invalid Type: {typ:#} must be of type '{operation:#}'")
            }
            ErrTypes::Expected(exp, found) => {
                format!("Wrong type: Expected {exp:#}, found {found:#}")
            }
            ErrTypes::ImplicitCast(type1, type2) => {
                format!("Cast error: Can not implicitly cast type {type1:#} into type {type2:#}")
            }
            ErrTypes::StackOverflow => format!("Stack overflow"), // TODO: impl this
        };
        let message = gen_message(data, Some((0, 0)), 0);
        println!("{message}");
        false
    }
}

#[allow(unused)]
pub mod runtime_types {
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
    pub struct Context {
        pub stack: Vec<Types>,
        pub call_stack: [CallStack; 255],
        pub stack_ptr: usize,
        pub registers: Registers,
        pub code: Vec<Instructions>,
        pub code_ptr: usize,
        pub garbage: Vec<usize>,
        pub heap: Vec<Vec<Types>>,
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
    }
    use std::fmt;
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
                }
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
        /// may expire at any time
        Heap(usize),
    }
    impl fmt::Display for PointerTypes {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                PointerTypes::Heap(n) => write!(f, "Heap({n})"),
                PointerTypes::Object => write!(f, "Object"),
                PointerTypes::Stack => write!(f, "Stack"),
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
        /// Repair: pointer | Repairs pointer in reg(<pointer>)
        // Repp, removed due to it being just Idx(0) will be replaced with static Indexing in the future
        /// Allocate: size_reg pointers_len | reserves <size> on heap and stores location in registers(<reg>)
        Alc(usize),
        /// Deallocate: | frees heap(<reg>)
        // Dalc, removed to make room for GC
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
                //Instructions::Dalc => "Deallocation",
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
                //Instructions::Repp => "RepirePointer",
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