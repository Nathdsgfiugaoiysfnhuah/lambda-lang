use std::collections::HashMap;
use std::fs;
use std::str;

enum ValueType {
    NUM,
    VEC,
}

struct Value {
    num: f64,
    vec: Vec<Value>,
    ty: ValueType,
}

struct FnDef {
    start: usize,
    argc: usize,
}

#[derive(Debug)]
enum Instruction {
    SUB,
    DIV,
    CALL,
    RET, // jump to last call
    CMP, // a, b => {-1,0,1} : {< = >}
    JZ,  // cond, adr
    ARR,
	SET,
}

#[derive(Debug)]
enum ByteCodePoint {
    c(Instruction),
    f(f64),
    i(i64),
	s(String),
}

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
    if tok == "fn" {
        let fn_name = next_tok(tokens, ptr);
        let argc = next_tok(tokens, ptr).parse::<usize>().unwrap();
        compiled.push(ByteCodePoint::c(Instruction::JZ));
        compiled.insert(compiled.len(), ByteCodePoint::i(0));
        let target = compiled.len();
        fns.last_mut().unwrap().insert(
            fn_name.clone(),
            FnDef {
                start: compiled.len() + 2, // + 2 to jump over the future adress value && ret instruction
                argc,
            },
        );
        fns.push(HashMap::new()); // create inner scope
        draw(tokens, ptr, fns, compiled); // get rid of body
        compiled.insert(target, ByteCodePoint::i(compiled.len() as i64 + 1)); // we now know how big the fn is, and so can insert code to skip to the end
        compiled.insert(target, ByteCodePoint::c(Instruction::RET)); // make sure we return out when called, push second to shift stuff over
        fns.pop(); // delete inner scope
        return;
    }
    if tok == "sub" {
		compiled.push(ByteCodePoint::c(Instruction::SUB));
		draw_many(tokens, ptr, 2, fns, compiled);
		return;
	}
	if tok == "div" {
		compiled.push(ByteCodePoint::c(Instruction::DIV));
		draw_many(tokens, ptr, 2, fns, compiled);
		return;
	}
	if tok == "do" {
		let count = next_tok(tokens, ptr).parse().unwrap();
		draw_many(tokens, ptr, count, fns, compiled);
		return;
	}
	if tok == "set" {
		compiled.push(ByteCodePoint::c(Instruction::SET));
		let var = next_tok(tokens, ptr); // what about &mut ?
		compiled.push(ByteCodePoint::s(var.to_string()));
		draw(tokens, ptr, fns, compiled); // get the value for it
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

fn main() {
    let content = fs::read("./source.src").unwrap();
    let s = match str::from_utf8(&content) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {e}"),
    };

    println!("source: \n{}", s);

    let mut tokens: Vec<String> = vec!["".to_string()];
    let mut escaped = false;
    let mut commented = false;
    for char in s.chars() {
        // println!("{escaped} {commented} {char} {chunks:?}");
        if escaped {
            *tokens.last_mut().unwrap() += &char.to_string();
            escaped = false;
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

    println!("{tokens:?}");

    let mut ptr = 0;
    let mut fn_stack: Vec<HashMap<String, FnDef>> = vec![HashMap::new()];
    let mut compiled: Vec<ByteCodePoint> = vec![];
    while ptr < tokens.len() {
        // treeification
        draw_many(&tokens, &mut ptr, 1, &mut fn_stack, &mut compiled);
    }
    println!("{compiled:?}");
}
