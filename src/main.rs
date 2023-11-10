use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

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
            let new_line = line
                .replace("“", "\"")
                .replace("”", "\"")
                .replace("’", "\'")
                .replace("‘", "\'");
            new_content.push_str(&new_line);
            new_content.push_str("\n");
        } else {
            // 处理 ![[*]] || #*
            if line.starts_with("!") || line.starts_with("#") {
                new_content.push_str(&line);
                new_content.push_str("\n");
                continue;
            }

            let line = replace_quotes(&line);

            // 删除所有空格
            let regex_space = Regex::new(r"\s+").unwrap();
            let no_space_string = regex_space.replace_all(&line, "");

            // 为中文字符串中的英文单词前后添加空格
            let regex_en = Regex::new(r"([a-zA-Z0-9\_\-\+]+)").unwrap();
            let en_space_string = regex_en.replace_all(&no_space_string, " $1 ");

            // 处理 “*” 表达式
            let regex_zh_qu = Regex::new(r"“(\s+)([^\s]*)(\s+)”").unwrap();
            let new_string = regex_zh_qu.replace_all(&en_space_string, "“$2”");

            // 处理 `*` 表达式
            let re = Regex::new(r"`(\s+)([^`]*)(\s+)`").unwrap();
            let result = re.replace_all(&new_string, " `$2` ");

            // 处理标点边界条件
            let line = result
                .replace(" ，", "，")
                .replace("， ", "，")
                .replace(" 。", "。")
                .replace("。 ", "。")
                .replace(" 、", "、")
                .replace("、 ", "、")
                .replace("： ", "：")
                .replace(" ：", "：")
                .replace(" ！", "！")
                .replace(" ）", "）")
                .replace("（ ", "（")
                .replace("/ ", "/")
                .replace(" /", "/")
                .replace("  ", " "); // `Event`*  *Trait -> `Event`* *Trait
            let line = line.trim();

            new_content.push_str(&line);
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
            ' ' => {
                if let Some('\"') = s.chars().nth(i + 1) {
                    if is_open_quote {
                        result.push('”'); // 中文右引号
                    } else {
                        result.push('“'); // 中文左引号
                    }
                    is_open_quote = !is_open_quote;
                } else {
                    result.push(c)
                }
            }
            '\"' => continue,
            _ => result.push(c),
        }
    }

    result
}
