use std::collections::HashMap;
use std::fs;
use std::str;

struct FnDef {
    start: usize,
    argc: usize,
}

#[derive(Debug)]
enum Instruction {
    SUB,
    ADD,
    DIV,
    MUL,
    CALL, // adr, argc, ...
    RET,  // jump to last call
    CMP,  // a, b => {-1,0,1} : {< = >}
    JZ,   // cond, adr
    ARR,
    SET,
    REF,
    VAL,
    PRINT,
}

enum FunctionRef {}

enum Value {
    Float(f64),
    Integer(i64),
    FunctionRef(FunctionRef),
}

#[derive(Debug)]
enum ByteCodePoint {
    Code(Instruction),
    Float(f64),
    Integer(i64),
    String(String),
}

// tokenisation

fn tokenise(src: &str) -> Vec<String> {
    let mut tokens: Vec<String> = vec!["".to_string()];
    let mut escaped = false;
    let mut commented = false;
    for char in src.chars() {
        if escaped {
            escaped = false;
            if commented {
                continue;
            }
            *tokens.last_mut().unwrap() += &char.to_string();
            continue;
        }
        if char == '\\' {
            escaped = true;
            continue;
        }
        if char == '/' {
            commented = !commented;
            continue;
        }
        if commented {
            continue;
        }
        if char == '\t' || char == ' ' || char == '\n' || char == '\r' {
            if tokens.last().unwrap() != "" {
                tokens.push("".to_string());
            }
            continue;
        }
        *tokens.last_mut().unwrap() += &char.to_string();
    }

    if tokens.last().unwrap().is_empty() {
        tokens.pop();
    }
    tokens
}

// ast building

fn next_tok<'a>(tokens: &'a [String], ptr: &mut usize) -> &'a String {
    *ptr += 1;
    &tokens[*ptr - 1]
}

fn draw(
    tokens: &[String],
    ptr: &mut usize,
    fns: &mut Vec<HashMap<String, FnDef>>,
    compiled: &mut Vec<ByteCodePoint>,
) {
    let tok = next_tok(tokens, ptr);
    match tok.as_str() {
        "fn" => {
            let fn_name = next_tok(tokens, ptr);
            let argc = next_tok(tokens, ptr).parse::<usize>().unwrap();
            compiled.push(ByteCodePoint::Code(Instruction::JZ));
            compiled.insert(compiled.len(), ByteCodePoint::Integer(0));
            let target = compiled.len();
            fns.last_mut().unwrap().insert(
                fn_name.clone(),
                FnDef {
                    start: compiled.len() + 1, // + 1 to jump over the future adress value instruction and land on good val
                    argc,
                },
            );
            fns.push(HashMap::new()); // create inner scope
            draw(tokens, ptr, fns, compiled); // get rid of body
            compiled.insert(target, ByteCodePoint::Integer(compiled.len() as i64 + 1)); // we now know how big the fn is, and so can insert code to skip to the end
            fns.pop(); // delete inner scope
        }
        "sub" => {
            compiled.push(ByteCodePoint::Code(Instruction::SUB));
            draw_many(tokens, ptr, 2, fns, compiled);
        }
        "add" => {
            compiled.push(ByteCodePoint::Code(Instruction::ADD));
            draw_many(tokens, ptr, 2, fns, compiled);
        }
        "div" => {
            compiled.push(ByteCodePoint::Code(Instruction::DIV));
            draw_many(tokens, ptr, 2, fns, compiled);
        }
        "mul" => {
            compiled.push(ByteCodePoint::Code(Instruction::MUL));
            draw_many(tokens, ptr, 2, fns, compiled);
        }
        "do" => {
            let count = next_tok(tokens, ptr).parse().unwrap(); // consider some kind of support for arbitrary do? i'm not sure it really makes sense though.
            draw_many(tokens, ptr, count, fns, compiled);
        }
        "set" => {
            compiled.push(ByteCodePoint::Code(Instruction::SET));
            draw_many(tokens, ptr, 2, fns, compiled); // get the value for it, does this even work though?
        }
        "ref" => {
            compiled.push(ByteCodePoint::Code(Instruction::REF));
            draw(tokens, ptr, fns, compiled);
        }
        "val" => {
            compiled.push(ByteCodePoint::Code(Instruction::VAL));
            draw(tokens, ptr, fns, compiled);
        }
        "print" => {
            compiled.push(ByteCodePoint::Code(Instruction::PRINT));
            draw(tokens, ptr, fns, compiled);
        }
        "arr" => {
            compiled.push(ByteCodePoint::Code(Instruction::ARR));
            let count = next_tok(tokens, ptr).parse().unwrap();
            draw_many(tokens, ptr, count, fns, compiled);
        }
        "comp" => {
            compiled.push(ByteCodePoint::Code(Instruction::CMP));
            draw_many(tokens, ptr, 2, fns, compiled);
        }
        "ret" => {
            compiled.push(ByteCodePoint::Code(Instruction::RET));
            draw(tokens, ptr, fns, compiled);
        }
		"if" => { // if cond fn ...
			// 1: JZ cond 3
			// 2: CALL fn, ...
			// 3: =>
            compiled.push(ByteCodePoint::Code(Instruction::JZ));
			let rembr = compiled.len();
			draw(tokens, ptr, fns, compiled);
			let func = next_tok(tokens, ptr);
			let mut argc: Option<&FnDef> = None; // TODO: functions as args dont work with this
            for scope in fns.iter().rev() {
                if let Some(v) = scope.get(func) {
                    argc = Some(v);
                    break;
                }
            }
            compiled.push(ByteCodePoint::Code(Instruction::CALL));
			draw_many(tokens, ptr, argc.unwrap().argc, fns, compiled);
			compiled.insert(rembr, ByteCodePoint::Integer(compiled.len() as i64));
		}
        _ => {
            let mut argc: Option<&FnDef> = None; // TODO: functions as args dont work with this
            for scope in fns.iter().rev() {
                if let Some(v) = scope.get(tok) {
                    argc = Some(v);
                    break;
                }
            }
            if let Some(def) = argc {
                compiled.push(ByteCodePoint::Code(Instruction::CALL));
                compiled.push(ByteCodePoint::Integer(def.start as i64));
                compiled.push(ByteCodePoint::Integer(def.argc as i64));
                draw_many(tokens, ptr, def.argc, fns, compiled);
                return;
            }
            if let Ok(val) = tok.parse::<i64>() {
                compiled.push(ByteCodePoint::Integer(val));
                return;
            }
            if let Ok(val) = tok.parse::<f64>() {
                compiled.push(ByteCodePoint::Float(val));
                return;
            }
            compiled.push(ByteCodePoint::String(tok.to_string()));
        }
    }
}

fn draw_many(
    tokens: &[String],
    ptr: &mut usize,
    count: usize,
    fns: &mut Vec<HashMap<String, FnDef>>,
    compiled: &mut Vec<ByteCodePoint>,
) {
    for _i in 0..count {
        draw(tokens, ptr, fns, compiled);
    }
}

fn get_bytecode(tokens: Vec<String>) -> Vec<ByteCodePoint> {
    let mut ptr = 0;
    let mut fn_stack: Vec<HashMap<String, FnDef>> = vec![HashMap::new()];
    let mut compiled: Vec<ByteCodePoint> = vec![];
    while ptr < tokens.len() {
        draw_many(&tokens, &mut ptr, 1, &mut fn_stack, &mut compiled);
    }
    compiled
}

// bytecode interpreter

fn eval(compiled: &Vec<ByteCodePoint>, instruction_pointer: &mut usize) {
    let instruction = &compiled[*instruction_pointer];
    match instruction {
        ByteCodePoint::Code(Instruction::JZ) => {
			*instruction_pointer+=1;
			// cond = eval();
		}
        _ => {}
    }
}

fn main() {
    let content = fs::read("./source.src").unwrap();
    let src = match str::from_utf8(&content) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {e}"),
    };

    println!("source: \n{}", src);

    let tokens = tokenise(src);
    println!("{tokens:?}");

    let compiled = get_bytecode(tokens);
    println!("{compiled:?}");

    let val_stack: Vec<HashMap<String, Value>> = vec![HashMap::new()];
    let call_stack: Vec<usize> = vec![];
    let mut instruction_pointer: usize = 0;
    eval(&compiled, &mut instruction_pointer);
}
