extern crate runtime;

use runtime::runtime::*;
use runtime::runtime::runtime_types::Context;
use runtime::runtime::runtime_types::Types;

pub struct Foo {
    pub a: i32,
    pub b: i32,
}

impl runtime::runtime::Library for Foo {
    fn init(&mut self, ctx: &mut Context) -> Result<Box<Self>, String> {
        return Ok(Box::new(Foo { a: 3, b: 0 }));
    }
    fn call(&mut self, id: usize, mem: (&mut Vec<Types>, &mut Vec<Vec<Types>>, &mut Vec<Vec<char>>)) -> Result<runtime_types::Types, runtime_error::ErrTypes> {
        match id {
            0 => {
                return Ok(runtime_types::Types::Int(self.a + self.b));
            }
            1 => {
                if let Types::Int(a) = mem.0[0] {
                    self.a = a;
                    return Ok(runtime_types::Types::Null);
                } else {
                    return Err(runtime_error::ErrTypes::Message("Invalid argument".to_owned()));
                }
            }
            // save string to memory
            2 => {
                mem.2.push("mem.0[0]".to_string().chars().collect());
            }
            _ => {unreachable!("Invalid function id")},
        }
        return Ok(runtime_types::Types::Null);
    }
    fn name(&self) -> String {
        return "Foo".to_owned();
    }
}

#[no_mangle]
pub fn init(ctx: &mut Context) -> Box<dyn Library> {
    return Box::new(Foo { a: 3, b: 0 });
}