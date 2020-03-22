use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::ops::{Add, Sub, Mul, Div};
use std::rc::Rc;
use std::time::Instant;

use serde::Deserialize;
use serde_json;

enum CompareOps {
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual
}

impl From<usize> for CompareOps {
    fn from(op: usize) -> Self {
        match op {
            0 => Self::LessThan,
            1 => Self::LessThanOrEqual,
            2 => Self::Equal,
            3 => Self::NotEqual,
            4 => Self::GreaterThan,
            5 => Self::GreaterThanOrEqual,
            _ => panic!("Unimplemented cmp_op")
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
enum Instruction {
    LoadConst(usize),
    StoreName(usize),
    LoadName(usize),
    DeleteName(usize),
    StoreFast(usize),
    LoadFast(usize),
    DeleteFast(usize),
    StoreGlobal(usize),
    LoadGlobal(usize),
    DeleteGlobal(usize),
    CompareOp(usize),
    JumpForward(usize),
    PopJumpIfTrue(usize),
    PopJumpIfFalse(usize),
    JumpIfTrueOrPop(usize),
    JumpIfFalseOrPop(usize),
    MakeFunction(usize),
    CallFunction(usize),
    JumpAbsolute(usize),
    ReturnValue,
    InplaceAdd,
    InplaceSubtract,
    InplaceMultiply,
    InplaceTrueDivide,
    InplaceFloorDivide,
    BinaryAdd,
    BinarySubtract,
    BinaryMultiply,
    BinaryTrueDivide,
    BinaryFloorDivide,
    Nop,
    PopTop,
    RotTwo,
    RotThree,
    RotFour,
    DupTop,
    DupTopTwo,
    UnaryPositive,
    UnaryNegative,

    Print,
}

#[derive(Clone, Debug, Deserialize)]
enum Value {
    Int(i32),
    Bool(bool),
    Float(f32),
    Str(String),
    Nonetype,
    Frame(Frame)
}

impl Default for Value {
    fn default() -> Self { Value::Nonetype }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(first), Value::Int(second)) => first == second,
            (Value::Bool(first), Value::Bool(second)) => first == second,
            (Value::Str(first), Value::Str(second)) => first == second,
            (Value::Float(first), Value::Float(second)) => first == second,

            (Value::Float(first), Value::Int(second)) | (Value::Int(second), Value::Float(first))  => (*second as f32).eq( first),
            (Value::Float(first), Value::Bool(second)) | (Value::Bool(second), Value::Float(first))  => first == &((*second as i32) as f32),
            (Value::Bool(first), Value::Int(second)) | (Value::Int(second), Value::Bool(first)) => (*first as i32).eq(second),

            _ => panic!("Unimplemented comparision between {:?} and {:?}", self, other)
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(first), Value::Int(second)) => first.partial_cmp(second),
            (Value::Bool(first), Value::Bool(second)) => first.partial_cmp(second),
            (Value::Str(first), Value::Str(second)) => first.partial_cmp(second),
            (Value::Float(first), Value::Float(second)) => first.partial_cmp(second),

            (Value::Float(first), Value::Int(second)) | (Value::Int(second), Value::Float(first))  => first.partial_cmp(&(*second as f32)),
            (Value::Float(first), Value::Bool(second)) | (Value::Bool(second), Value::Float(first))  => first.partial_cmp(&((*second as i32) as f32)),
            (Value::Bool(first), Value::Int(second)) | (Value::Int(second), Value::Bool(first)) => (*first as i32).partial_cmp(second),

            _ => panic!("Unimplemented comparision between {:?} and {:?}", self, other)
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(first), Value::Int(second)) => Value::Int(first + second),
            (Value::Float(first), Value::Float(second)) => Value::Float(first + second),
            (Value::Bool(first), Value::Bool(second)) => Value::Int((*first as i32) + (*second as i32)),
            (Value::Str(first), Value::Str(second)) => Value::Str(first.clone() + second),
            (Value::Float(first), Value::Int(second)) | (Value::Int(second), Value::Float(first))  => Value::Float(first + (*second as f32)),
            (Value::Bool(first), Value::Int(second)) | (Value::Int(second), Value::Bool(first)) => Value::Int((*first as i32) + second),

            _ => panic!("Unimplemented 'add' operation between {:?} and {:?}", self, rhs)
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(first), Value::Int(second)) => Value::Int(first - second),
            (Value::Float(first), Value::Float(second)) => Value::Float(first - second),
            (Value::Bool(first), Value::Bool(second)) => Value::Int((*first as i32) - (*second as i32)),
            (Value::Float(first), Value::Int(second)) | (Value::Int(second), Value::Float(first))  => Value::Float(first - (*second as f32)),
            (Value::Bool(first), Value::Int(second)) | (Value::Int(second), Value::Bool(first)) => Value::Int((*first as i32) - second),

            _ => panic!("Unimplemented 'add' operation between {:?} and {:?}", self, rhs)
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(first), Value::Int(second)) => Value::Int(first * second),
            (Value::Float(first), Value::Float(second)) => Value::Float(first * second),
            (Value::Bool(first), Value::Bool(second)) => Value::Int((*first as i32) * (*second as i32)),
            (Value::Str(first), Value::Int(second)) | (Value::Int(second), Value::Str(first)) => {
                let mut res = first.clone();
                for _ in 1..*second {
                    res += first;
                };
                Value::Str(res)
            },
            (Value::Float(first), Value::Int(second)) | (Value::Int(second), Value::Float(first))  => Value::Float(first * (*second as f32)),
            (Value::Bool(first), Value::Int(second)) | (Value::Int(second), Value::Bool(first)) => Value::Int((*first as i32) * second),

            _ => panic!("Unimplemented 'add' operation between {:?} and {:?}", self, rhs)
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Int(first), Value::Int(second)) => Value::Float((*first as f32) / (*second as f32)),
            (Value::Float(first), Value::Float(second)) => Value::Float(first / second),
            (Value::Bool(first), Value::Bool(second)) => Value::Float((*first as i32) as f32 / (*second as i32) as f32),
            (Value::Float(first), Value::Int(second)) | (Value::Int(second), Value::Float(first))  => Value::Float(first / (*second as f32)),
            (Value::Bool(first), Value::Int(second)) | (Value::Int(second), Value::Bool(first)) => Value::Float((*first as i32) as f32 / (*second as i32) as f32),

            _ => panic!("Unimplemented 'add' operation between {:?} and {:?}", self, rhs)
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Frame {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    co_names: Vec<Rc<String>>,
    co_varnames: Vec<Rc<String>>,

    #[serde(default)]
    stack: Vec<Value>,
    #[serde(default)]
    index: usize,
    #[serde(default)]
    globals: HashMap<Rc<String>, Value>,
    #[serde(default)]
    locals: HashMap<Rc<String>, Value>,
    #[serde(default)]
    return_value: Box<Value>,
    #[serde(default)]
    depth: usize,
}

impl Frame {
    fn run(&mut self) {
        while let Some(instruction) = self.instructions.get(self.index) {
            match *instruction {
                Instruction::LoadConst(arg) => self.load_const(arg),
                Instruction::StoreName(arg) => self.store_name(arg),
                Instruction::LoadName(arg) => self.load_name(arg),
                Instruction::DeleteName(arg) => self.delete_name(arg),
                Instruction::StoreFast(arg) => self.store_fast(arg),
                Instruction::LoadFast(arg) => self.load_fast(arg),
                Instruction::DeleteFast(arg) => self.delete_fast(arg),
                Instruction::StoreGlobal(arg) => self.store_global(arg),
                Instruction::LoadGlobal(arg) => self.load_global(arg),
                Instruction::DeleteGlobal(arg) => self.delete_global(arg),
                Instruction::CompareOp(arg) => self.compare_op(arg),
                Instruction::JumpForward(arg) => { self.index += arg / 2 + 1; },
                Instruction::PopJumpIfTrue(arg) => self.pop_jump_if_true(arg),
                Instruction::PopJumpIfFalse(arg) => self.pop_jump_if_false(arg),
                Instruction::JumpIfTrueOrPop(arg) => self.jump_if_true_or_pop(arg),
                Instruction::JumpIfFalseOrPop(arg) => self.jump_if_false_or_pop(arg),
                Instruction::JumpAbsolute(arg) =>  { self.index = arg / 2; },
                Instruction::MakeFunction(arg) => self.make_function(arg),
                Instruction::CallFunction(arg) => self.call_function(arg),
                Instruction::ReturnValue => self.return_value(),
                Instruction::InplaceAdd => self.add(),
                Instruction::InplaceSubtract => self.subtract(),
                Instruction::InplaceMultiply => self.multiply(),
                Instruction::InplaceTrueDivide => self.true_divide(),
                Instruction::InplaceFloorDivide => self.floor_divide(),
                Instruction::BinaryAdd => self.add(),
                Instruction::BinarySubtract => self.subtract(),
                Instruction::BinaryMultiply => self.multiply(),
                Instruction::BinaryTrueDivide => self.true_divide(),
                Instruction::BinaryFloorDivide => self.floor_divide(),
                Instruction::Nop => { self.index += 1; },
                Instruction::PopTop => self.pop_top(),
                Instruction::RotTwo => self.rot_two(),
                Instruction::RotThree => self.rot_three(),
                Instruction::RotFour => self.rot_four(),
                Instruction::DupTop => self.dup_top(),
                Instruction::DupTopTwo => self.dup_top_two(),
                Instruction::UnaryPositive => { self.index += 1 },
                Instruction::UnaryNegative => self.unary_negative(),

                Instruction::Print => self.print(),
            };
        };
    }

    fn load_const(&mut self, arg: usize) {
        self.stack.push(self.constants[arg].clone());

        self.index += 1;
    }

    fn store_name(&mut self, arg: usize) {
        self.locals.insert(Rc::clone(&self.co_names[arg]), self.stack.pop().unwrap());

        self.index += 1;
    }

    fn load_name(&mut self, arg: usize) {
        self.stack.push(self.locals[&self.co_names[arg]].clone());

        self.index += 1;
    }

    fn delete_name(&mut self, arg: usize) {
        self.locals.remove(&self.co_names[arg]);

        self.index += 1;
    }

    fn store_fast(&mut self, arg: usize) {
        self.locals.insert(Rc::clone(&self.co_varnames[arg]), self.stack.pop().unwrap());

        self.index += 1;
    }

    fn load_fast(&mut self, arg: usize) {
        self.stack.push(self.locals.get(&self.co_varnames[arg]).unwrap().clone());

        self.index += 1;
    }

    fn delete_fast(&mut self, arg: usize) {
        self.locals.remove(&self.co_varnames[arg]);

        self.index += 1;
    }

    fn store_global(&mut self, arg: usize) {
        self.globals.insert(Rc::clone(&self.co_names[arg]), self.stack.pop().unwrap());

        self.index += 1;
    }

    fn load_global(&mut self, arg: usize) {
        self.stack.push(self.globals.get(&self.co_names[arg]).unwrap().clone());

        self.index += 1;
    }

    fn delete_global(&mut self, arg: usize) {
        self.globals.remove(&self.co_names[arg]);

        self.index += 1;
    }

    fn compare_op(&mut self, arg: usize) {
        let second_var = self.stack.pop().unwrap();
        let first_var = self.stack.pop().unwrap();

        self.stack.push(Value::Bool(
            match CompareOps::from(arg) {
                CompareOps::LessThan => first_var < second_var,
                CompareOps::LessThanOrEqual => first_var <= second_var,
                CompareOps::Equal => first_var == second_var,
                CompareOps::NotEqual => first_var != second_var,
                CompareOps::GreaterThan => first_var > second_var,
                CompareOps::GreaterThanOrEqual => first_var >= second_var,
            }
        ));

        self.index += 1;
    }

    fn pop_jump_if_true(&mut self, arg: usize) {
        if let Value::Bool(result) = self.stack.pop().unwrap() {
            if result {
                self.index = arg / 2;
            } else {
                self.index += 1;
            }
        } else {
            panic!("Invalid `Value` passed to compare");
        }
    }

    fn pop_jump_if_false(&mut self, arg: usize) {
        if let Value::Bool(result) = self.stack.pop().unwrap() {
            if !result {
                self.index = arg / 2;
            } else {
                self.index += 1;
            }
        } else {
            panic!("Invalid `Value` passed to compare");
        }
    }

    fn jump_if_true_or_pop(&mut self, arg: usize) {
        if let Value::Bool(result) = self.stack.last().unwrap() {
            if *result {
                self.index = arg / 2;
            } else {
                self.stack.pop();

                self.index += 1;
            }
        } else {
            panic!("Invalid `Value` passed to compare");
        }
    }

    fn jump_if_false_or_pop(&mut self, arg: usize) {
        if let Value::Bool(result) = self.stack.last().unwrap() {
            if !(*result) {
                self.index = arg / 2;
            } else {
                self.stack.pop();

                self.index += 1;
            }
        } else {
            panic!("Invalid `Value` passed to compare");
        }
    }

    fn make_function(&mut self, arg: usize) {
        if arg != 0 {
            panic!("Unimplemented function flag")
        }

        if let (Value::Str(_), Value::Frame(frame)) = (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
            self.stack.push( Value::Frame(frame));
        } else {
            panic!("Wrong types for TOS and TOS1")
        }

        self.index += 1;
    }

    fn call_function(&mut self, arg: usize) {
        if let Value::Frame(mut frame) = self.stack.remove(self.stack.len() - arg - 1) {
            for i in 0..arg {
                frame.locals.insert(Rc::clone(&frame.co_varnames[frame.co_varnames.len() - i - 1]), self.stack.pop().unwrap());
            };
            // These were not supposed to be clones but lifetimes are hard
            if self.depth == 0 {
                frame.globals = self.locals.clone();
            } else {
                frame.globals = self.globals.clone();
            }
            frame.depth += self.depth + 1;
            frame.run();
            self.stack.push(*frame.return_value);
        } else {
            panic!("Wrong type for TOS");
        }

        self.index += 1;
    }

    fn return_value(&mut self) {
        self.return_value = Box::new(self.stack.pop().unwrap());

        self.index = self.instructions.len();
    }

    fn add(&mut self) {
        let mut result = self.stack.pop().unwrap();
        result = self.stack.pop().unwrap() + result;
        self.stack.push(result);

        self.index += 1;
    }

    fn subtract(&mut self) {
        let mut result = self.stack.pop().unwrap();
        result = self.stack.pop().unwrap() - result;
        self.stack.push(result);

        self.index += 1;
    }

    fn multiply(&mut self) {
        let mut result = self.stack.pop().unwrap();
        result = self.stack.pop().unwrap() * result;
        self.stack.push(result);

        self.index += 1;
    }

    fn true_divide(&mut self) {
        let mut result = self.stack.pop().unwrap();
        result = self.stack.pop().unwrap() / result;
        self.stack.push(result);

        self.index += 1;
    }

    fn floor_divide(&mut self) {
        let mut result = self.stack.pop().unwrap();
        result = self.stack.pop().unwrap() / result;
        if let Value::Float(result) = result {
            self.stack.push(Value::Int(result as i32));
        } else {
            self.stack.push(result);
        }

        self.index += 1;
    }

    fn pop_top(&mut self) {
        self.stack.pop();

        self.index += 1;
    }

    fn rot_two(&mut self) {
        let last_pos = self.stack.len() - 1;
        self.stack.swap(last_pos, last_pos - 1);

        self.index += 1;
    }

    fn rot_three(&mut self) {
        let last_pos = self.stack.len() - 1;
        self.stack.swap(last_pos, last_pos - 1);
        self.stack.swap(last_pos - 1, last_pos - 2);

        self.index += 1;
    }

    fn rot_four(&mut self) {
        let last_pos = self.stack.len() - 1;
        self.stack.swap(last_pos, last_pos - 1);
        self.stack.swap(last_pos - 1, last_pos - 2);
        self.stack.swap(last_pos - 2, last_pos - 3);

        self.index += 1;
    }

    fn dup_top(&mut self) {
        self.stack.push(self.stack.last().unwrap().clone());

        self.index += 1;
    }

    fn dup_top_two(&mut self) {
        self.stack.push(self.stack[self.stack.len() - 1].clone());
        self.stack.insert(self.stack.len() - 3, self.stack[self.stack.len() - 3].clone());

        self.index += 1;
    }

    fn unary_negative(&mut self) {
        let negative = Value::Int(0) - self.stack.pop().unwrap();
        self.stack.push(negative);

        self.index += 1;
    }
    
    fn create_print_frame() -> Frame {
        Frame {
            instructions: vec![
                Instruction::LoadFast(0),
                Instruction::Print
            ],
            constants: vec![Value::Str(String::from("to_print"))],
            co_names: vec![],
            co_varnames: vec![Rc::new(String::from("to_print"))],
            stack: vec![],
            index: 0,
            globals: Default::default(),
            locals: Default::default(),
            return_value: Box::new(Value::Nonetype),
            depth: 0
        }
    }

    fn print(&mut self) {
        match self.stack.pop().unwrap() {
            Value::Int(val) => println!("{}", val),
            Value::Bool(val) => println!("{}", val),
            Value::Float(val) => println!("{}", val),
            Value::Str(val) => println!("{}", val),
            Value::Nonetype => println!("None"),
            Value::Frame(val) => println!("{:#?}", val)
        }

        self.index += 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut frame: Frame = serde_json::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
    frame.locals.insert(Rc::new(String::from("print")), Value::Frame(Frame::create_print_frame()));

    let now = Instant::now();
    frame.run();
    println!("Running Took: {:?}", now.elapsed());
}
