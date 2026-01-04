use ratatui::style::{Color, Style};
use std::path::Path;
use syntect::parsing::{SyntaxReference, SyntaxSet};

// Background color for code view
pub const CODE_BG: Color = Color::Rgb(40, 40, 45); // Dark gray background

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        Self { syntax_set }
    }

    pub fn detect_syntax(&self, file_path: &Path) -> &SyntaxReference {
        self.syntax_set
            .find_syntax_for_file(file_path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }

    pub fn highlight_line(&self, line: &str, _syntax: &SyntaxReference) -> Vec<(Style, String)> {
        // Use simple token-based highlighting with custom colors
        let mut result = Vec::new();

        // Keywords for common languages
        let keywords = [
            "function", "const", "let", "var", "return", "if", "else", "for", "while",
            "fn", "pub", "impl", "struct", "enum", "trait", "use", "mod",
            "def", "class", "import", "from", "async", "await",
            "int", "string", "bool", "void", "char", "float", "double",
        ];

        let mut current_pos = 0;
        let chars: Vec<char> = line.chars().collect();

        while current_pos < chars.len() {
            let remaining = &chars[current_pos..];

            // Check for strings
            if remaining[0] == '"' || remaining[0] == '\'' {
                let quote = remaining[0];
                let mut end_pos = 1;
                while end_pos < remaining.len() && remaining[end_pos] != quote {
                    if remaining[end_pos] == '\\' && end_pos + 1 < remaining.len() {
                        end_pos += 2;
                    } else {
                        end_pos += 1;
                    }
                }
                if end_pos < remaining.len() {
                    end_pos += 1;
                }
                let string_text: String = remaining[..end_pos].iter().collect();
                result.push((
                    Style::default().fg(Color::Rgb(152, 195, 121)), // Green for strings
                    string_text,
                ));
                current_pos += end_pos;
                continue;
            }

            // Check for comments
            if current_pos + 1 < chars.len() && remaining[0] == '/' && remaining[1] == '/' {
                let comment: String = remaining.iter().collect();
                result.push((
                    Style::default().fg(Color::Rgb(120, 120, 130)), // Gray for comments
                    comment,
                ));
                break;
            }

            // Check for numbers
            if remaining[0].is_ascii_digit() {
                let mut end_pos = 0;
                while end_pos < remaining.len() && (remaining[end_pos].is_ascii_digit() || remaining[end_pos] == '.') {
                    end_pos += 1;
                }
                let number: String = remaining[..end_pos].iter().collect();
                result.push((
                    Style::default().fg(Color::Rgb(209, 154, 102)), // Orange for numbers
                    number,
                ));
                current_pos += end_pos;
                continue;
            }

            // Check for keywords/identifiers
            if remaining[0].is_alphabetic() || remaining[0] == '_' {
                let mut end_pos = 0;
                while end_pos < remaining.len() && (remaining[end_pos].is_alphanumeric() || remaining[end_pos] == '_') {
                    end_pos += 1;
                }
                let word: String = remaining[..end_pos].iter().collect();

                let is_keyword = keywords.contains(&word.as_str());
                let color = if is_keyword {
                    Color::Rgb(198, 120, 221) // Purple for keywords
                } else {
                    Color::Rgb(220, 220, 255) // White for identifiers
                };

                result.push((Style::default().fg(color), word));
                current_pos += end_pos;
                continue;
            }

            // Operators and punctuation
            let ch = remaining[0];
            let color = match ch {
                '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '|' | '^' | '%' => {
                    Color::Rgb(86, 182, 194) // Cyan for operators
                }
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    Color::Rgb(255, 198, 109) // Yellow for brackets
                }
                _ => Color::Rgb(220, 220, 255), // White for others
            };

            result.push((Style::default().fg(color), ch.to_string()));
            current_pos += 1;
        }

        result
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
