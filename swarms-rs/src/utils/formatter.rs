use colored::*;
use std::sync::OnceLock;

// MARKDOWN RENDERING RULES
// ========================
//
// The formatter supports the following markdown elements:
//
// HEADERS:
// - `# Title` → H1: Green bordered box with black background
// - `## Section` → H2: Blue background with white text, underlined
// - `### Subsection` → H3: Cyan background with black text, underlined
// - `#### Subsubsection` → H4: Magenta text, no background
//
// TEXT FORMATTING:
// - `**bold text**` → Bold white text on blue background
// - `*italic text*` → Italic cyan text
// - Mixed `**bold** and regular` → Inline bold formatting
//
// LISTS:
// - `- item` → Bullet point with • marker
// - `1. item` → Numbered list (not yet implemented)
//
// CODE:
// - `` `code` `` → Inline code with black background
// - ```rust\ncode\n``` → Code block with syntax highlighting
// - ```\ncode\n``` → Code block without language
//
// QUOTES:
// - `> quote` → Blockquote with yellow italic text
//
// SEPARATORS:
// - `---` → Horizontal rule with dotted line
//
// LINKS:
// - `[text](url)` → Not yet implemented
//
// TABLES:
// - `| col1 | col2 |` → Not yet implemented
//
// NOT RENDERED:
// - Table of contents `[text](#anchor)` → Renders as plain text
// - HTML tags → Renders as plain text
// - Complex nested structures → May break rendering

/// A simple, essential formatter for console output
pub struct Formatter {
    markdown_enabled: bool,
}

impl Formatter {
    /// Creates a new Formatter instance
    pub fn new(markdown: bool) -> Self {
        Self {
            markdown_enabled: markdown,
        }
    }

    /// Basic syntax highlighting for code
    fn highlight_syntax(&self, line: &str, language: &str) -> String {
        if !self.markdown_enabled {
            return format!("  {}", line);
        }

        let mut highlighted = String::new();
        let words: Vec<&str> = line.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            let highlighted_word = if word.starts_with("//") || word.starts_with("#") {
                // Comments
                format!("{}", word.italic().dimmed().green())
            } else if word.starts_with('"') && word.ends_with('"') {
                // String literals
                format!("{}", word.bold().yellow())
            } else if word.starts_with('\'') && word.ends_with('\'') {
                // Char literals
                format!("{}", word.bold().yellow())
            } else if word.ends_with('(') || word.ends_with(')') {
                // Function calls
                let func_name = word.trim_end_matches('(');
                format!("{}{}", func_name.bold().cyan(), "(".white())
            } else if word.ends_with('{') || word.ends_with('}') {
                // Braces
                format!("{}", word.bold().magenta())
            } else if word.ends_with(';') {
                // Statements ending with semicolon
                let stmt = word.trim_end_matches(';');
                format!("{}{}", stmt.white(), ";".bold().magenta())
            } else if self.is_keyword(word, language) {
                // Keywords (language-specific)
                format!("{}", word.bold().blue())
            } else if word.starts_with("self") || word.starts_with("Self") {
                // Self references
                format!("{}", word.bold().green())
            } else if word.chars().all(|c| c.is_uppercase() || c == '_') && word.len() > 1 {
                // Constants (all caps)
                format!("{}", word.bold().yellow())
            } else if word.starts_with("&") || word.starts_with("*") {
                // References and pointers
                let ref_part = word.chars().take_while(|&c| matches!(c, '&' | '*')).collect::<String>();
                let var_part = word.trim_start_matches(['&', '*']);
                format!("{}{}", ref_part.bold().red(), var_part.white())
            } else if word.contains("::") {
                // Module paths
                let parts: Vec<&str> = word.split("::").collect();
                let mut formatted = String::new();
                for (j, part) in parts.iter().enumerate() {
                    if j > 0 {
                        formatted.push_str(&format!("{}", "::".bold().magenta()));
                    }
                    formatted.push_str(&format!("{}", part.bold().cyan()));
                }
                formatted
            } else {
                // Default
                format!("{}", word.white())
            };
            
            highlighted.push_str(&highlighted_word);
            if i < words.len() - 1 {
                highlighted.push(' ');
            }
        }
        
        format!("  {}", highlighted)
    }

    /// Check if a word is a keyword for the given language
    fn is_keyword(&self, word: &str, language: &str) -> bool {
        match language {
            "rust" | "rs" => {
                matches!(word, "let" | "mut" | "fn" | "pub" | "impl" | "struct" | "enum" | "trait" | 
                         "use" | "mod" | "async" | "await" | "return" | "if" | "else" | 
                         "match" | "for" | "while" | "loop" | "break" | "continue" | "const" | "static" |
                         "type" | "where" | "in" | "as" | "ref" | "move" | "unsafe" | "extern")
            },
            "python" | "py" => {
                matches!(word, "def" | "class" | "import" | "from" | "as" | "if" | "elif" | "else" |
                         "for" | "while" | "try" | "except" | "finally" | "with" | "return" | "yield" |
                         "pass" | "break" | "continue" | "True" | "False" | "None" | "self" | "lambda")
            },
            "javascript" | "js" => {
                matches!(word, "function" | "var" | "let" | "const" | "if" | "else" | "for" | "while" |
                         "try" | "catch" | "finally" | "return" | "break" | "continue" | "switch" | "case" |
                         "default" | "class" | "extends" | "super" | "new" | "this" | "typeof" | "instanceof")
            },
            _ => false
        }
    }

    /// Prints a basic panel to the console
    pub fn print_panel(&self, content: &str, title: &str, _style: &str) {
        let content = if content.is_empty() { "No content to display" } else { content };

        if self.markdown_enabled {
            self.print_markdown_panel(content, title);
        } else {
            self.print_basic_panel(content, title);
        }
    }

    /// Prints a basic panel without markdown
    fn print_basic_panel(&self, content: &str, title: &str) {
        const WIDTH: usize = 80;
        const BORDER: &str = "══════════════════════════════════════════════════════════════════════════════════════════════════════════";
        const CORNER_TL: &str = "╭";
        const CORNER_TR: &str = "╮";
        const CORNER_BL: &str = "╰";
        const CORNER_BR: &str = "╯";
        const SIDE_L: &str = "│";
        const SIDE_R: &str = "│";

        let title_line = format!(" {} ", title);
        let padding = (WIDTH - 4 - title_line.len()) / 2;
        let title_border = format!("{}{}{}", 
            "─".repeat(padding), 
            title_line, 
            "─".repeat(WIDTH - 4 - padding - title_line.len())
        );

        println!("\n{}{}{}", CORNER_TL.green().bold(), &BORDER[..WIDTH-2].green().dimmed(), CORNER_TR.green().bold());
        println!("{}{}{}", SIDE_L.green().bold(), title_border.green().bold(), SIDE_R.green().bold());
        println!("{}{}{}", SIDE_L.green().bold(), &BORDER[..WIDTH-2].green().dimmed(), SIDE_R.green().bold());
        
        // Print content with proper indentation
        for line in content.lines() {
            let padded_line = format!("{:<width$}", line, width = WIDTH - 2);
            println!("{}{}{}", SIDE_L.green().dimmed(), padded_line.white(), SIDE_R.green().dimmed());
        }
        
        println!("{}{}{}", SIDE_L.green().bold(), &BORDER[..WIDTH-2].green().dimmed(), SIDE_R.green().bold());
        println!("{}{}{}\n", CORNER_BL.green().bold(), &BORDER[..WIDTH-2].green().dimmed(), CORNER_BR.green().bold());
    }

    /// Prints content as markdown with syntax highlighting
    pub fn print_markdown(&self, content: &str, title: &str, _border_style: &str) {
        if self.markdown_enabled {
            self.print_markdown_panel(content, title);
        } else {
            self.print_basic_panel(content, title);
        }
    }

    /// Prints a markdown panel with enhanced styling
    fn print_markdown_panel(&self, content: &str, title: &str) {
        if content.trim().is_empty() {
            return;
        }

        const WIDTH: usize = 80;
        const BORDER: &str = "══════════════════════════════════════════════════════════════════════════════════════════════════════════";
        const CORNER_TL: &str = "╭";
        const CORNER_TR: &str = "╮";
        const CORNER_BL: &str = "╰";
        const CORNER_BR: &str = "╯";
        const SIDE_L: &str = "│";
        const SIDE_R: &str = "│";

        let title_line = format!(" {} ", title);
        let padding = (WIDTH - 4 - title_line.len()) / 2;
        let title_border = format!("{}{}{}", 
            "─".repeat(padding), 
            title_line, 
            "─".repeat(WIDTH - 4 - padding - title_line.len())
        );

        println!("\n{}{}{}", CORNER_TL.blue().bold(), &BORDER[..WIDTH-2].blue().dimmed(), CORNER_TR.blue().bold());
        println!("{}{}{}", SIDE_L.blue().bold(), title_border.blue().bold(), SIDE_R.blue().bold());
        println!("{}{}{}", SIDE_L.blue().bold(), &BORDER[..WIDTH-2].blue().dimmed(), SIDE_R.blue().bold());
        
        // Print content with proper indentation
        for line in content.lines() {
            let padded_line = format!("{:<width$}", line, width = WIDTH - 2);
            println!("{}{}{}", SIDE_L.blue().dimmed(), padded_line.white(), SIDE_R.blue().dimmed());
        }
        
        println!("{}{}{}", SIDE_L.blue().bold(), &BORDER[..WIDTH-2].blue().dimmed(), SIDE_R.blue().bold());
        println!("{}{}{}\n", CORNER_BL.blue().bold(), &BORDER[..WIDTH-2].blue().dimmed(), CORNER_BR.blue().bold());
    }

    /// Renders markdown text with proper formatting and colors
    pub fn render_markdown(&self, text: &str) {
        if !self.markdown_enabled {
            // Fast path: just print the text without formatting
            println!("{}", text);
            return;
        }

        self.render_markdown_internal(text, false);
    }

    /// Internal markdown rendering function that can be used for both standalone and bordered content
    fn render_markdown_internal(&self, text: &str, in_border: bool) {
        let mut in_code_block = false;
        let mut code_language = String::new();
        
        for line in text.lines() {
            let line = line.trim();

            if let Some(header) = line.strip_prefix("### ") {
                if in_border {
                    self.render_h3_header_bordered(header);
                } else {
                    self.render_h3_header(header, in_border);
                }
            } else if let Some(header) = line.strip_prefix("#### ") {
                if in_border {
                    self.render_h4_header_bordered(header);
                } else {
                    self.render_h4_header(header);
                }
            } else if let Some(header) = line.strip_prefix("## ") {
                if in_border {
                    self.render_h2_header_bordered(header);
                } else {
                    self.render_h2_header(header, in_border);
                }
            } else if let Some(header) = line.strip_prefix("# ") {
                if in_border {
                    self.render_h1_header_bordered(header);
                } else {
                    self.render_h1_header(header, in_border);
                }
            } else if line.contains("**") {
                if in_border {
                    self.render_mixed_bold_text_bordered(line);
                } else {
                    self.render_mixed_bold_text(line);
                }
            } else if line.starts_with("**") && line.ends_with("**") {
                if in_border {
                    self.render_bold_text_bordered(line);
                } else {
                    self.render_bold_text(line);
                }
            } else if let Some(bullet) = line.strip_prefix("- ") {
                if in_border {
                    self.render_bullet_point_bordered(bullet);
                } else {
                    self.render_bullet_point(bullet);
                }
            } else if line.starts_with("---") {
                if in_border {
                    self.render_horizontal_rule_bordered();
                } else {
                    self.render_horizontal_rule();
                }
            } else if line.is_empty() {
                if in_border {
                    self.render_empty_line_bordered();
                } else {
                    self.render_empty_line();
                }
            } else if line == "```" {
                in_code_block = false;
                code_language.clear();
                if in_border {
                    self.render_code_block_end_bordered();
                } else {
                    self.render_code_block_end();
                }
            } else if let Some(stripped) = line.strip_prefix("```") {
                in_code_block = true;
                code_language = stripped.to_string();
                if in_border {
                    self.render_code_block_start_bordered(&code_language);
                } else {
                    self.render_code_block_start(&code_language);
                }
            } else if in_code_block {
                if in_border {
                    self.render_code_content_bordered(line, &code_language);
                } else {
                    self.render_code_content(line, &code_language);
                }
            } else if line.starts_with("`") && line.ends_with("`") {
                if in_border {
                    self.render_inline_code_bordered(line);
                } else {
                    self.render_inline_code(line);
                }
            } else if let Some(quote) = line.strip_prefix("> ") {
                if in_border {
                    self.render_blockquote_bordered(quote);
                } else {
                    self.render_blockquote(quote);
                }
            } else if line.starts_with("*") && line.ends_with("*") {
                if in_border {
                    self.render_italic_text_bordered(line);
                } else {
                    self.render_italic_text(line);
                }
            } else if in_border {
                self.render_regular_text_bordered(line);
            } else {
                self.render_regular_text(line);
            }
        }
    }

    /// Renders H1 headers (Document title with premium styling)
    fn render_h1_header(&self, header: &str, in_border: bool) {
        if !in_border {
            println!();
        }
        println!("{}", format!("╭─ {} ─╮", "─".repeat(header.len())).green().bold());
        println!("{}", format!("│ {} │", header).bold().green().on_black());
        println!("{}", format!("╰─ {} ─╯", "─".repeat(header.len())).green().bold());
        if !in_border {
            println!();
        }
    }

    /// Renders H1 headers within bordered content
    fn render_h1_header_bordered(&self, header: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let padded_header = format!("{:<width$}", format!("  {}  ", header).bold().green().on_black(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_header, VERTICAL.yellow().dimmed());
    }

    /// Renders H2 headers (Major section headers with prominent styling)
    fn render_h2_header(&self, header: &str, in_border: bool) {
        if !in_border {
            println!();
        }
        println!("{}", format!("  {}  ", header).bold().blue().on_white());
        println!("{}", format!("  {}  ", "═".repeat(header.len())).blue().dimmed());
        if !in_border {
            println!();
        }
    }

    /// Renders H3 headers (Section headers with enhanced styling)
    fn render_h3_header(&self, header: &str, in_border: bool) {
        if !in_border {
            println!();
        }
        println!("{}", format!("  {}  ", header).bold().cyan().on_black());
        println!("{}", format!("  {}  ", "─".repeat(header.len())).cyan().dimmed());
        if !in_border {
            println!();
        }
    }

    /// Renders H4 headers (Subsection headers with subtle styling)
    fn render_h4_header(&self, header: &str) {
        println!("{}", format!("    {}  ", header).bold().magenta());
    }

    /// Renders mixed bold text within a line
    fn render_mixed_bold_text(&self, line: &str) {
        let parts: Vec<&str> = line.split("**").collect();
        let mut formatted = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i % 2 == 1 {
                // Bold part
                formatted.push_str(&format!("{}", part.bold().white().on_blue()));
            } else {
                // Regular part
                formatted.push_str(part);
            }
        }
        println!("{}", format!("  {}", formatted).white());
    }

    /// Renders bold text (entire line)
    fn render_bold_text(&self, line: &str) {
        let bold_text = &line[2..line.len() - 2];
        println!("{}", format!("  {}  ", bold_text).bold().white().on_blue());
    }

    /// Renders italic text
    fn render_italic_text(&self, line: &str) {
        let italic_text = &line[1..line.len()-1];
        println!("{}", format!("  {}  ", italic_text).italic().cyan());
    }

    /// Renders bullet points with styled markers
    fn render_bullet_point(&self, bullet: &str) {
        println!("{}", format!("  • {}  ", bullet).white());
    }

    /// Renders horizontal rules with enhanced styling
    fn render_horizontal_rule(&self) {
        println!("{}", "┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄".dimmed());
    }

    /// Renders empty lines
    fn render_empty_line(&self) {
        println!();
    }

    /// Renders code block start
    fn render_code_block_start(&self, language: &str) {
        if language.is_empty() {
            println!("{}", "┌─────────────────────────────────────────────────────────────────────────────────────────┐".dimmed());
        } else {
            println!("{}", format!("┌─ {} ─", language).dimmed());
        }
    }

    /// Renders code block end
    fn render_code_block_end(&self) {
        println!("{}", "└─────────────────────────────────────────────────────────────────────────────────────────┘".dimmed());
    }

    /// Renders code content within code block with syntax highlighting
    fn render_code_content(&self, line: &str, language: &str) {
        println!("{}", self.highlight_syntax(line, language));
    }

    /// Renders inline code with styling
    fn render_inline_code(&self, line: &str) {
        let code = &line[1..line.len()-1];
        println!("{}", format!(" `{}` ", code).white().on_black().bold());
    }

    /// Renders blockquotes with enhanced styling
    fn render_blockquote(&self, quote: &str) {
        println!("{}", format!("  > {}  ", quote).italic().dimmed().yellow());
    }

    /// Renders regular text with enhanced paragraph styling
    fn render_regular_text(&self, line: &str) {
        if !line.is_empty() {
            println!("{}", format!("  {}", line).white());
        }
    }

    // BORDERED VERSIONS OF ALL RENDERING FUNCTIONS

    /// Renders H2 headers within bordered content
    fn render_h2_header_bordered(&self, header: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let padded_header = format!("{:<width$}", format!("  {}  ", header).bold().blue().on_white(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_header, VERTICAL.yellow().dimmed());
        let underline = format!("{:<width$}", format!("  {}  ", "═".repeat(header.len())).blue().dimmed(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), underline, VERTICAL.yellow().dimmed());
    }

    /// Renders H3 headers within bordered content
    fn render_h3_header_bordered(&self, header: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let padded_header = format!("{:<width$}", format!("  {}  ", header).bold().cyan().on_black(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_header, VERTICAL.yellow().dimmed());
        let underline = format!("{:<width$}", format!("  {}  ", "─".repeat(header.len())).cyan().dimmed(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), underline, VERTICAL.yellow().dimmed());
    }

    /// Renders H4 headers within bordered content
    fn render_h4_header_bordered(&self, header: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let padded_header = format!("{:<width$}", format!("    {}  ", header).bold().magenta(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_header, VERTICAL.yellow().dimmed());
    }

    /// Renders mixed bold text within bordered content
    fn render_mixed_bold_text_bordered(&self, line: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let parts: Vec<&str> = line.split("**").collect();
        let mut formatted = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i % 2 == 1 {
                // Bold part
                formatted.push_str(&format!("{}", part.bold().white().on_blue()));
            } else {
                // Regular part
                formatted.push_str(part);
            }
        }
        let padded_line = format!("{:<width$}", format!("  {}", formatted).white(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_line, VERTICAL.yellow().dimmed());
    }

    /// Renders bold text within bordered content
    fn render_bold_text_bordered(&self, line: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let bold_text = &line[2..line.len() - 2];
        let padded_bold = format!("{:<width$}", format!("  {}  ", bold_text).bold().white().on_blue(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_bold, VERTICAL.yellow().dimmed());
    }

    /// Renders italic text within bordered content
    fn render_italic_text_bordered(&self, line: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let italic_text = &line[1..line.len()-1];
        let padded_italic = format!("{:<width$}", format!("  {}  ", italic_text).italic().cyan(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_italic, VERTICAL.yellow().dimmed());
    }

    /// Renders bullet points within bordered content
    fn render_bullet_point_bordered(&self, bullet: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let padded_bullet = format!("{:<width$}", format!("  • {}  ", bullet).white(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_bullet, VERTICAL.yellow().dimmed());
    }

    /// Renders horizontal rules within bordered content
    fn render_horizontal_rule_bordered(&self) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let rule = "┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄".dimmed();
        let padded_rule = format!("{:<width$}", rule, width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_rule, VERTICAL.yellow().dimmed());
    }

    /// Renders empty lines within bordered content
    fn render_empty_line_bordered(&self) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        println!("{}{}{}", VERTICAL.yellow().dimmed(), " ".repeat(WIDTH - 2), VERTICAL.yellow().dimmed());
    }

    /// Renders code block start within bordered content
    fn render_code_block_start_bordered(&self, language: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let code_text = if language.is_empty() {
            "┌─ Code Block ─".dimmed()
        } else {
            format!("┌─ {} ─", language).dimmed()
        };
        let padded_code = format!("{:<width$}", format!("  {}  ", code_text), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_code, VERTICAL.yellow().dimmed());
    }

    /// Renders code block end within bordered content
    fn render_code_block_end_bordered(&self) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let padded_code = format!("{:<width$}", format!("  {}  ", "└─ End Code Block ─").dimmed(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_code, VERTICAL.yellow().dimmed());
    }

    /// Renders code content within bordered content
    fn render_code_content_bordered(&self, line: &str, language: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let highlighted = self.highlight_syntax(line, language);
        let padded_code = format!("{:<width$}", highlighted, width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_code, VERTICAL.yellow().dimmed());
    }

    /// Renders inline code within bordered content
    fn render_inline_code_bordered(&self, line: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let code = &line[1..line.len()-1];
        let padded_code = format!("{:<width$}", format!("  `{}`  ", code).white().on_black().bold(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_code, VERTICAL.yellow().dimmed());
    }

    /// Renders blockquotes within bordered content
    fn render_blockquote_bordered(&self, quote: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        let padded_quote = format!("{:<width$}", format!("  > {}  ", quote).italic().dimmed().yellow(), width = WIDTH - 2);
        println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_quote, VERTICAL.yellow().dimmed());
    }

    /// Renders regular text within bordered content
    fn render_regular_text_bordered(&self, line: &str) {
        const VERTICAL: &str = "│";
        const WIDTH: usize = 80;
        if !line.is_empty() {
            let padded_line = format!("{:<width$}", format!("  {}", line).white(), width = WIDTH - 2);
            println!("{}{}{}", VERTICAL.yellow().dimmed(), padded_line, VERTICAL.yellow().dimmed());
        }
    }

    /// Renders a section header with consistent styling
    pub fn render_section_header(&self, title: &str) {
        if self.markdown_enabled {
            println!("{}", format!("[SECTION] {}", title).bold().cyan());
            println!("{}", "─".repeat(60).dimmed());
        } else {
            println!("[SECTION] {}", title);
        }
    }

    /// Renders a success message
    pub fn render_success(&self, message: &str) {
        if self.markdown_enabled {
            println!("{}", format!("[SUCCESS] {}", message).green());
        } else {
            println!("[SUCCESS] {}", message);
        }
    }

    /// Renders an error message
    pub fn render_error(&self, message: &str) {
        if self.markdown_enabled {
            println!("{}", format!("[ERROR] {}", message).red());
        } else {
            println!("[ERROR] {}", message);
        }
    }

    /// Renders a warning message
    pub fn render_warning(&self, message: &str) {
        if self.markdown_enabled {
            println!("{}", format!("[WARNING] {}", message).yellow());
        } else {
            println!("[WARNING] {}", message);
        }
    }

    /// Renders an info message
    pub fn render_info(&self, message: &str) {
        if self.markdown_enabled {
            println!("{}", format!("[INFO] {}", message).cyan());
        } else {
            println!("[INFO] {}", message);
        }
    }

    /// Renders workflow completion
    pub fn render_workflow_completion(&self, workflow_name: &str) {
        if self.markdown_enabled {
            println!("{}", format!("[COMPLETE] {} finished successfully!", workflow_name).bold().green());
        } else {
            println!("[COMPLETE] {} finished successfully!", workflow_name);
        }
    }

    /// Renders agent output with enhanced borders and styling
    pub fn render_agent_output(&self, agent_name: &str, content: &str) {
        if self.markdown_enabled {
            self.render_agent_output_tui(agent_name, content);
        } else {
            self.render_agent_output_basic(agent_name, content);
        }
    }

    /// Renders agent output using TUI for proper border rendering
    fn render_agent_output_tui(&self, agent_name: &str, content: &str) {
        // For now, fall back to the enhanced version to avoid TUI complexity
        // We'll implement a simpler approach with better Unicode borders
        self.render_agent_output_enhanced_improved(agent_name, content);
    }

    /// Renders agent output with improved Unicode borders that work better across terminals
    fn render_agent_output_enhanced_improved(&self, agent_name: &str, content: &str) {
        // Use simpler, more compatible Unicode characters
        const TOP_LEFT: &str = "┌";
        const TOP_RIGHT: &str = "┐";
        const BOTTOM_LEFT: &str = "└";
        const BOTTOM_RIGHT: &str = "┘";
        const HORIZONTAL: &str = "─";
        const TOP_T: &str = "┬";
        
        let width = 80;
        let title = format!(" {} ", agent_name);
        let title_width = title.len();
        let padding = (width - title_width - 2) / 2;
        
        // Top border with title
        let top_line = format!(
            "{}{}{}{}{}{}{}",
            TOP_LEFT,
            HORIZONTAL.repeat(padding),
            TOP_T,
            title,
            TOP_T,
            HORIZONTAL.repeat(width - padding - title_width - 4),
            TOP_RIGHT
        );
        
        println!("\n{}", top_line.yellow().bold());
        
        // Use the modular markdown rendering system for content
        self.render_markdown_internal(content, true);
        
        // Bottom border
        let bottom_line = format!("{}{}{}", BOTTOM_LEFT, HORIZONTAL.repeat(width - 2), BOTTOM_RIGHT);
        println!("{}\n", bottom_line.yellow().bold());
    }

    /// Renders agent output with basic formatting (no markdown)
    fn render_agent_output_basic(&self, agent_name: &str, content: &str) {
        const WIDTH: usize = 80;
        const BORDER: &str = "══════════════════════════════════════════════════════════════════════════════════════════════════════════";
        const CORNER_TL: &str = "╭";
        const CORNER_TR: &str = "╮";
        const CORNER_BL: &str = "╰";
        const CORNER_BR: &str = "╯";
        const SIDE_L: &str = "│";
        const SIDE_R: &str = "│";

        let title_line = format!(" {} ", agent_name);
        let padding = (WIDTH - 4 - title_line.len()) / 2;
        let title_border = format!("{}{}{}", 
            "─".repeat(padding), 
            title_line, 
            "─".repeat(WIDTH - 4 - padding - title_line.len())
        );

        println!("\n{}{}{}", CORNER_TL.yellow().bold(), &BORDER[..WIDTH-2].yellow().dimmed(), CORNER_TR.yellow().bold());
        println!("{}{}{}", SIDE_L.yellow().bold(), title_border.yellow().bold(), SIDE_R.yellow().bold());
        println!("{}{}{}", SIDE_L.yellow().bold(), &BORDER[..WIDTH-2].yellow().dimmed(), SIDE_R.yellow().bold());
        
        // Print content with proper indentation
        for line in content.lines() {
            let padded_line = format!("{:<width$}", line, width = WIDTH - 2);
            println!("{}{}{}", SIDE_L.yellow().dimmed(), padded_line.white(), SIDE_R.yellow().dimmed());
        }
        
        println!("{}{}{}", SIDE_L.yellow().bold(), &BORDER[..WIDTH-2].yellow().dimmed(), SIDE_R.yellow().bold());
        println!("{}{}{}\n", CORNER_BL.yellow().bold(), &BORDER[..WIDTH-2].yellow().dimmed(), CORNER_BR.yellow().bold());
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new(false) // Disable markdown by default for better performance
    }
}

// Static formatter instance for convenience functions
static FORMATTER: OnceLock<Formatter> = OnceLock::new();

fn get_formatter() -> &'static Formatter {
    FORMATTER.get_or_init(|| Formatter::new(false))
}

// Convenience functions for easy access - now use static instance
pub fn render_markdown(text: &str) {
    get_formatter().render_markdown(text);
}

pub fn render_section_header(title: &str) {
    get_formatter().render_section_header(title);
}

pub fn render_success(message: &str) {
    get_formatter().render_success(message);
}

pub fn render_error(message: &str) {
    get_formatter().render_error(message);
}

pub fn render_info(message: &str) {
    get_formatter().render_info(message);
}

pub fn render_warning(message: &str) {
    get_formatter().render_warning(message);
}

pub fn render_workflow_completion(workflow_name: &str) {
    get_formatter().render_workflow_completion(workflow_name);
}

/// Renders agent output with enhanced borders and styling
pub fn render_agent_output(agent_name: &str, content: &str) {
    get_formatter().render_agent_output(agent_name, content);
}

/// Initialize the formatter (for compatibility with existing code)
pub fn init_formatter(markdown: bool) {
    let _ = FORMATTER.set(Formatter::new(markdown));
}
