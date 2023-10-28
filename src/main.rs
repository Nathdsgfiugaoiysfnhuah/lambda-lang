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

fn next_tok<'a>(tokens: &'a [String], ptr: &mut usize) -> &'a String {
    *ptr += 1;
    &tokens[*ptr - 1]
}

fn draw(tokens: &[String], ptr: &mut usize, fns: &mut Vec<HashMap<String, FnDef>>) {
    let tok = next_tok(tokens, ptr);
    if tok == "fn" {
        let fn_name = next_tok(tokens, ptr);
        let argc = usize::from_str_radix(next_tok(tokens, ptr), 10).unwrap();
        fns.last_mut().unwrap().insert(
            fn_name.clone(),
            FnDef {
                start: *ptr + 1,
                argc,
            },
        );
        fns.push(HashMap::new());
    }
}

fn draw_many(
    tokens: &[String],
    ptr: &mut usize,
    count: usize,
    fns: &mut Vec<HashMap<String, FnDef>>,
) {
    for i in 0..count {
        draw(tokens, ptr, fns);
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
    let mut tree: TreeNode;
    while ptr < tokens.len() {
        // treeification
        &draw_many(&tokens, &mut ptr, 1, &mut fn_stack);
    }
}
