use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use clap::Parser;


/// Obsidian md format
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the markdown file to format
    #[arg(short, long)]
    file: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.file {
        None => {
            println!("No markdown file specified. Please specify your markdown file.");
            Ok(())
        }
        Some(program_name) => run_file(&program_name),
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // 读取文件内容
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let mut new_content = String::new();

    // 修改文件内容
    for line in content.lines() {
        if !is_chinese(&line) {
            let new_line = line.replace("“", "\"").replace("”", "\"").replace("’", "\'").replace("‘", "\'");
            new_content.push_str(&new_line);
            new_content.push_str("\n");
        } else {
            let new_line = replace_quotes(line);
            new_content.push_str(&new_line);
            new_content.push_str("\n");
        }
    }

    // 将修改后的文件内容写回到文件中
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(new_content.as_bytes())?;

    Ok(())
}

fn is_chinese(s: &str) -> bool {
    s.trim().chars().any(|c| c >= '\u{4E00}' && c <= '\u{9FFF}')
}

fn replace_quotes(s: &str) -> String {
    let mut result = String::new();
    let mut is_open_quote = false;

    for (i, c) in s.chars().enumerate() {
        match c {
          ' ' => continue,
            '\"' => {
              if s.chars().nth(i-1).unwrap() == ' ' {
                if is_open_quote {
                    result.push('”'); // 中文右引号
                } else {
                    result.push('“'); // 中文左引号
                }
                is_open_quote = !is_open_quote;
              }
            },
            _ => result.push(c),
        }
    }

    result
}

