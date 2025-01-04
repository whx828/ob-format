use std::fs;
use std::path::PathBuf;
use std::error::Error;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(value_parser = clap::value_parser!(PathBuf))]
    path: PathBuf,
}

// 新增：代码块保护结构
#[derive(Debug)]
struct TextSegment {
    content: String,
    is_code_block: bool,
}

// 新增：代码块处理规则
struct CodeBlockProtector;
impl CodeBlockProtector {
    fn split_text(&self, text: &str) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut current_text = String::new();
        let mut line_iter = text.lines().peekable();

        while let Some(line) = line_iter.next() {
            if line.trim_start().starts_with("```") {
                // 如果有累积的普通文本，先添加到段落中
                if !current_text.is_empty() {
                    segments.push(TextSegment {
                        content: current_text,
                        is_code_block: false,
                    });
                    current_text = String::new();
                }

                // 开始收集代码块
                let mut code_block = String::from(line);
                code_block.push('\n');

                // 继续收集直到找到结束的 ```
                while let Some(code_line) = line_iter.next() {
                    code_block.push_str(code_line);
                    code_block.push('\n');
                    if code_line.trim_start().starts_with("```") {
                        break;
                    }
                }

                // 添加代码块段落
                segments.push(TextSegment {
                    content: code_block,
                    is_code_block: true,
                });
            } else {
                current_text.push_str(line);
                if line_iter.peek().is_some() {
                    current_text.push('\n');
                }
            }
        }

        // 添加最后剩余的文本
        if !current_text.is_empty() {
            segments.push(TextSegment {
                content: current_text,
                is_code_block: false,
            });
        }

        segments
    }
}

trait TypographyRule {
    fn apply(&self, text: &str) -> String;
}

// 中英文标点转换规则
struct QuotationMarkConverter;
impl TypographyRule for QuotationMarkConverter {
    fn apply(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut in_english = false;
        
        for (i, &c) in chars.iter().enumerate() {
            // 检查是否在英文上下文中
            if i > 0 {
                let prev = chars[i - 1];
                if is_english(prev) || prev.is_ascii_digit() {
                    in_english = true;
                } else if is_chinese(prev) {
                    in_english = false;
                }
            }
            
            let converted = if in_english {
                match c {
                    '“' => '\"',
                    '”' => '\"',
                    '‘' => '\'',
                    '’' => '\'',
                    _ => c
                }
            } else {
                c
            };
            result.push(converted);
        }
        result
    }
}

// 行首尾空格处理规则
struct LineSpaceTrimmer;
impl TypographyRule for LineSpaceTrimmer {
    fn apply(&self, text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// 中文标点空格处理规则
struct ChinesePunctuationSpacing;
impl TypographyRule for ChinesePunctuationSpacing {
    fn apply(&self, text: &str) -> String {
        let punctuation = ['，', '。', '、', '？', '！'];
        let mut result = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let current = chars[i];
            
            if punctuation.contains(&current) {
                // 如果当前字符是标点符号，移除前面的空格（如果有的话）
                if result.ends_with(' ') {
                    result.pop();
                }
                result.push(current);
                
                // 跳过标点符号后的空格
                if i + 1 < chars.len() && chars[i + 1] == ' ' {
                    i += 1;
                }
            } else {
                result.push(current);
            }
            
            i += 1;
        }
        
        result
    }
}

// 反引号包围文本的空格处理规则
struct BacktickSpacing;
impl TypographyRule for BacktickSpacing {
    fn apply(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let chinese_punct = ['，', '。', '、', '？', '！', '“', '”', '‘', '’'];
        
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '`' {
                // 找到配对的反引号
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '`' {
                    i += 1;
                }
                let end = i;
                
                // 处理反引号前的空格
                if start > 0 && !chinese_punct.contains(&chars[start - 1]) {
                    if !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
                
                // 添加反引号和其中的内容
                for j in start..=end {
                    if j < chars.len() {
                        result.push(chars[j]);
                    }
                }
                
                // 处理反引号后的空格
                if end < chars.len() - 1 && !chinese_punct.contains(&chars[end + 1]) {
                    result.push(' ');
                }
            } else {
                result.push(chars[i]);
            }
            i += 1;
        }
        result
    }
}

// 原有的中英文空格规则
struct ChineseEnglishSpacing;
impl TypographyRule for ChineseEnglishSpacing {
    fn apply(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);
        let chars: Vec<char> = text.chars().collect();
        
        for (i, &c) in chars.iter().enumerate() {
            if i > 0 {
                let prev = chars[i - 1];
                if (is_chinese(prev) && is_english(c)) || (is_english(prev) && is_chinese(c)) {
                    result.push(' ');
                }
            }
            result.push(c);
        }
        result
    }
}

// 原有的中文数字空格规则
struct ChineseNumberSpacing;
impl TypographyRule for ChineseNumberSpacing {
    fn apply(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);
        let chars: Vec<char> = text.chars().collect();
        
        for (i, &c) in chars.iter().enumerate() {
            if i > 0 {
                let prev = chars[i - 1];
                if (is_chinese(prev) && c.is_ascii_digit()) || (prev.is_ascii_digit() && is_chinese(c)) {
                    result.push(' ');
                }
            }
            result.push(c);
        }
        result
    }
}

struct Typesetter {
    rules: Vec<Box<dyn TypographyRule>>,
}

impl Typesetter {
    fn new() -> Self {
        let rules: Vec<Box<dyn TypographyRule>> = vec![
            Box::new(LineSpaceTrimmer), // 首先处理行首尾空格
            Box::new(QuotationMarkConverter), // 然后转换引号
            Box::new(ChineseEnglishSpacing), // 添加中英文空格
            Box::new(ChineseNumberSpacing), // 添加中文数字空格
            Box::new(ChinesePunctuationSpacing), // 处理中文标点周围的空格
            Box::new(BacktickSpacing), // 最后处理反引号
        ];
        Self { rules }
    }

    fn process(&self, text: &str) -> String {
        // 首先使用 CodeBlockProtector 分割文本
        let protector = CodeBlockProtector;
        let segments = protector.split_text(text);
        
        // 对每个段落分别处理
        let processed_segments: Vec<String> = segments
            .into_iter()
            .map(|segment| {
                if segment.is_code_block {
                    // 代码块保持原样
                    segment.content
                } else {
                    // 普通文本应用所有规则
                    let mut content = segment.content;
                    for rule in &self.rules {
                        content = rule.apply(&content);
                    }
                    content
                }
            })
            .collect();

        // 合并处理后的段落
        processed_segments.join("")
    }
}

fn is_chinese(c: char) -> bool {
    matches!(c,
        '\u{4e00}'..='\u{9fff}' | // CJK统一汉字
        '\u{3400}'..='\u{4dbf}' | // CJK扩展A
        '\u{20000}'..='\u{2a6df}' // CJK扩展B
    )
}

fn is_english(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let content = fs::read_to_string(&cli.path)
        .map_err(|e| format!("无法读取文件 '{}': {}", cli.path.display(), e))?;

    let typesetter = Typesetter::new();
    let processed = typesetter.process(&content);

    fs::write(&cli.path, processed)
        .map_err(|e| format!("无法写入文件 '{}': {}", cli.path.display(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quotation_converter() {
        let rule = QuotationMarkConverter;
        assert_eq!(rule.apply("This is a “test” text"), "This is a \"test\" text");
        assert_eq!(rule.apply("This is a ‘test’ text"), "This is a 'test' text");
    }

    #[test]
    fn test_line_space_trimmer() {
        let rule = LineSpaceTrimmer;
        assert_eq!(rule.apply("  hello  \n  world  "), "hello\nworld");
    }

    #[test]
    fn test_chinese_punctuation_spacing() {
        let rule = ChinesePunctuationSpacing;
        assert_eq!(rule.apply("你好 ，世界"), "你好，世界");
        assert_eq!(rule.apply("测试。 测试"), "测试。测试");
    }

    #[test]
    fn test_backtick_spacing() {
        let rule = BacktickSpacing;
        assert_eq!(rule.apply("在`code`中"), "在 `code` 中");
        assert_eq!(rule.apply("文本，`code`。"), "文本，`code`。");
    }
    #[test]
    fn test_code_block_protection() {
        let typesetter = Typesetter::new();
        let input = r#"这是一段中文 text 混合。

```rust
fn main() {
    println!("Hello， World!");  // 保持原样，包括中文标点
}
```

这是另一段 text 。"#;

        let output = typesetter.process(input);
        
        // 验证代码块保持不变
        assert!(output.contains("```rust\nfn main() {\n    println!(\"Hello， World!\");  // 保持原样，包括中文标点\n}\n```"));
        
        // 验证其他部分正常处理
        assert!(output.contains("这是一段中文 text 混合。"));
        assert!(output.contains("这是另一段 text。"));
    }

    #[test]
    fn test_multiple_code_blocks() {
        let typesetter = Typesetter::new();
        let input = r#"开始文本。

```python
print("Hello， World!")
```

中间文本。

```rust
let x = "test"；
```

结束文本。"#;

        let output = typesetter.process(input);
        assert!(output.contains("```python\nprint(\"Hello， World!\")\n```"));
        assert!(output.contains("```rust\nlet x = \"test\"；\n```"));
    }
}
