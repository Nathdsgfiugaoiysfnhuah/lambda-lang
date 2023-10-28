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

struct TreeNode {
	children: Vec<TreeNode>,
}

fn draw<'a>(tokens: &'a [String], ptr: &mut usize) {
	let tok = tokens[*ptr];
	*ptr+=1;
	if tok == "fn" {
		
	}
}

fn draw_many<'a>(tokens: &'a [String], ptr: &mut usize, count: usize) {
    for i in 0..count {
		draw(tokens, ptr);
	}
}

fn main() {
    let content = fs::read("./source.cat").unwrap();
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
    let mut fn_stack: Vec<HashMap<String, FnDef>>;
	let mut tree: TreeNode;
    while ptr < tokens.len() { // treeification
        let tok = &draw(&tokens, &mut ptr, 1)[0];
		if tok == "fn" {
			let fn_name = 
		}
    }
}
