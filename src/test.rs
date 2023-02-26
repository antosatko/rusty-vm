
pub mod test {
    use crate::runtime::runtime_types::{Context, Instructions::*, Types::*, *};

    const ID: usize = 4;
    pub fn test_init(id: Option<usize>, context: &mut Context) -> bool {
        let test_id = if let Some(num) = id { num } else { ID };
        println!("Running test {test_id}");
        match test_id {
            0 => {
                context.stack = vec![Int(0)];
                context.code = vec![End];
                true
            }
            // heap test
            1 => {
                context.stack = vec![
                    Int(1), // value to write
                    Null, // pointer placeholder
                    Usize(5), // size of object
                    Bool(true), // second value
                    Usize(3), // position of second value in object
                    Usize(4), // new size for realloc
                ];
                context.code = vec![
                    // stack
                    Res(3, 0),
                    // allocating size Usize(5)
                    Rd(1, 0),
                    Alc(0),
                    // writing pointer on stack
                    Wr(2, 0),
                    // writing to pointer
                    Swap(0, POINTER_REG),
                    Rd(3, 0), // value
                    Wrp(0),
                    // writing to pointer[Usize(3)]
                    Rdc(4, 0), // index
                    Idx(0),
                    Rdc(3, 0), // value
                    Wrp(0),
                    // resizing to Usize(4)
                    Rdc(5, 0), // size
                    Rd(2, POINTER_REG), // pointer
                    RAlc(0),
                    // free
                    //Dalc,
                    End
                ];
                true
            }
            // function swap
            2 => {
                context.stack = vec![
                    Int(3), // value 1
                    Int(7), // value 2
                    Bool(true), 
                    Null, // unused value
                    Int(0), // index
                    Int(50), // max
                    Int(1), // step
                ];
                context.code = vec![
                    Res(7, 0), // main stack
                    Goto(15), // skip function declaration to the main code
                    // function swap stack[bool, (ptr, ptr), tmp] -> bool
                    // write tmp value of pointer1
                    Rd(3, POINTER_REG),
                    Rdp(0),
                    Wr(1, 0),
                    // write pointer2 to pointer1
                    Rd(2, POINTER_REG),
                    Rdp(0), // value of pointer2
                    Rd(3, POINTER_REG),
                    Wrp(0),
                    // write tmp on pointer2
                    Rd(1, 0),
                    Rd(2, POINTER_REG),
                    Wrp(0),
                    // return true
                    Rdc(2, RETURN_REG),
                    Ufrz,
                    Ret,
                    // calling
                    Rd(1 + 3, GENERAL_REG1),
                    Res(4, 0), // function args stack
                    Frz,
                    Ptr(3 + 4 + 3),
                    Wr(3, GENERAL_REG1),
                    Ptr(4 + 4 + 3),
                    Wr(2, GENERAL_REG1),
                    Jump(2),
                    Rd(3, GENERAL_REG1),
                    Rd(1, GENERAL_REG2),
                    Add,
                    Wr(3, GENERAL_REG1),
                    Rd(2, GENERAL_REG2),
                    Less,
                    Brnc(15, 30),
                    End
                ];
                true
            }
            // function swap (optimized)
            3 => {
                context.stack = vec![
                    Int(3), // value 1
                    Int(7), // value 2
                    Bool(true), // return value
                    Int(0), // index
                    Int(50), // max
                    Int(1), // step
                ];
                context.code = vec![
                    Res(6, 0),
                    Goto(10),
                    // function swap registers[gen3: ptr, ptr:ptr]
                    Rdp(GENERAL_REG1), // load first value
                    // load second value
                    Swap(GENERAL_REG3, POINTER_REG),
                    Rdp(GENERAL_REG2),
                    Wrp(GENERAL_REG1), // write first value
                    // write second value
                    Swap(GENERAL_REG3, POINTER_REG),
                    Wrp(GENERAL_REG2),
                    Rdc(2, RETURN_REG), // return value
                    Back,
                    // calling
                    Ptr(2 + 3),
                    Swap(GENERAL_REG1, GENERAL_REG3),
                    Ptr(3 + 3),
                    Swap(GENERAL_REG1, POINTER_REG),
                    Jump(2),
                    Rd(3, GENERAL_REG1),
                    Rd(1, GENERAL_REG2),
                    Add,
                    Wr(3, GENERAL_REG1),
                    Rd(2, GENERAL_REG2),
                    Less,
                    Brnc(10, 22),
                    End
                ];
                true
            }
            // memory goes brrrrrrrrr
            4 => {
                context.stack = vec![
                    Pointer(1, PointerTypes::Object),
                    Usize(1), // size allocated on each iteration; low for safety measures
                    Int(0), // index
                    Int(1), // step
                    Int(60), // range
                    Null, // placeholder for heap pointer
                ];
                context.code = vec![
                    Res(6, 1),
                    Rdc(1, GENERAL_REG2), // size
                    Alc(GENERAL_REG2),
                    Move(GENERAL_REG1, POINTER_REG),
                    Rd(4, GENERAL_REG1),
                    Rd(3, GENERAL_REG2),
                    Add,
                    Wr(4, GENERAL_REG1),
                    Rd(2, GENERAL_REG2),
                    Less,
                    Brnc(1, 11),
                    Debug(POINTER_REG),
                    Rdc(1, GENERAL_REG2), // size
                    Rdc(1, GENERAL_REG1), // size
                    SweepUnoptimized,
                    Alc(GENERAL_REG2),
                    Sub,
                    Idx(GENERAL_REG1),
                    Wrp(GENERAL_REG2),
                    End
                ];
                true
            }
            5 => {
                context.stack = vec![
                    Usize(1),
                    Null,
                    Int(70),
                ];
                context.code = vec![
                    Res(3, 0),
                    Rd(3, GENERAL_REG1),
                    Alc(GENERAL_REG1),
                    Wr(2, GENERAL_REG1),
                    Rd(3, GENERAL_REG1),
                    Rd(3, GENERAL_REG2),
                    Add,
                    Rd(2, POINTER_REG),
                    RAlc(GENERAL_REG1),
                    Idx(GENERAL_REG2),
                    Wrp(GENERAL_REG1),
                    Alc(GENERAL_REG2),
                    Move(GENERAL_REG1, GENERAL_REG3),
                    Move(GENERAL_REG1, POINTER_REG),
                    Rd(3, GENERAL_REG1),
                    Sub,
                    Idx(GENERAL_REG1),
                    Rdp(GENERAL_REG1),
                    Debug(GENERAL_REG1),
                    Rd(2, POINTER_REG),
                    //Dalc,
                    Move(GENERAL_REG3, POINTER_REG),
                    Rd(3, GENERAL_REG1),
                    Sub,
                    Idx(GENERAL_REG1),
                    Rd(1, GENERAL_REG1),
                    Wrp(GENERAL_REG1),
                    End,
                ];
                true
            }
            6 => {
                context.stack = vec![
                    Usize(1),
                    Null,
                    Int(70),
                    Usize(0),
                ];
                context.code = vec![
                    Res(3, 0),
                    Rdc(0, GENERAL_REG1),
                    Alc(GENERAL_REG1),
                    Wr(2, POINTER_REG),
                    Rdc(3, GENERAL_REG1),
                    Idx(GENERAL_REG1),
                    Rd(1, GENERAL_REG1),
                    Wrp(GENERAL_REG1),
                    End,
                ];
                true
            }
            _ => {
                context.stack = vec![Int(0)];
                context.code = vec![End];
                println!("Test id: {test_id} not found.");
                true
            }
        }
    }
}
