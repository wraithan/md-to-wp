extern crate comrak;
extern crate typed_arena;

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use typed_arena::Arena;
use comrak::{parse_document, format_html, ComrakOptions};
use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};
use std::env;

fn main() {
    let input_file_path = env::args().nth(1).expect("filename required");

    let mut output_file_path = PathBuf::from(&input_file_path);
    output_file_path.set_extension("html");

    let mut input_file = File::open(&input_file_path).unwrap();
    let mut output_file = File::create(&output_file_path).unwrap();
    let mut raw_markdown = String::new();
    input_file.read_to_string(&mut raw_markdown).unwrap();

    let arena = Arena::new();

    let root = parse_document(&arena, &raw_markdown, &ComrakOptions::default());

    iter_nodes(root, &|node| {
        let ref mut value = node.data.borrow_mut().value;

        let new_value = match value {
            &mut NodeValue::CodeBlock(ref codeblock) => {
                let formatted = code_to_html(&codeblock.literal, &codeblock.info);
                NodeValue::HtmlBlock(
                    NodeHtmlBlock{
                        block_type: 0,
                        literal: formatted
                    }
                )
            }
            _ => value.to_owned(),
        };
        *value = new_value;
    });

    let html_string = format_html(root, &ComrakOptions::default());
    write!(output_file, "{}", html_string).unwrap();
    println!("{} -> {}", input_file_path, output_file_path.to_str().expect("could not convert output path to str"));
}

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
    where F : Fn(&'a AstNode<'a>) {
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub fn code_to_html(input: &String, lang: &String) -> String {
    let mut pyg_proc = Command::new("pygmentize")
                           .args(&["-f", "html"])
                           .args(&["-O", "noclasses"])
                           .args(&["-l", &lang])
                           .stdin(Stdio::piped())
                           .stdout(Stdio::piped())
                           .stderr(Stdio::inherit())
                           .spawn()
                           .expect("pygmentize failed to launch error should be above");
    
    if let Some(ref mut stdin) = pyg_proc.stdin {
        write!(stdin, "{}", &input).unwrap();
    }

    let pyg_output = pyg_proc.wait_with_output()
                             .expect("pygmentize wait failed?");
    String::from_utf8(pyg_output.stdout).expect("could not read pygmentize output")
}