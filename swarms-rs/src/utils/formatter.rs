use colored::*;
use console::Term;

/// A simple, essential formatter for console output
pub struct Formatter {
    console: Term,
    markdown_enabled: bool,
}

impl Formatter {
    /// Creates a new Formatter instance
    pub fn new(markdown: bool) -> Self {
        Self {
            console: Term::stdout(),
            markdown_enabled: markdown,
        }
    }

    /// Prints a basic panel to the console
    pub fn print_panel(&mut self, content: &str, title: &str, _style: &str) {
        let content = if content.is_empty() { "No content to display" } else { content };

        if self.markdown_enabled {
            self.print_markdown_panel(content, title);
        } else {
            self.print_basic_panel(content, title);
        }
    }

    /// Prints a basic panel without markdown
    fn print_basic_panel(&self, content: &str, title: &str) {
        let width = 80;
        let border = "═".repeat(width - 2);
        let title_line = format!(" {} ", title);
        let padding = (width - 4 - title_line.len()) / 2;
        let title_border = format!("{}{}{}", 
            "═".repeat(padding), 
            title_line, 
            "═".repeat(width - 4 - padding - title_line.len())
        );

        // Document-style corners and sides
        let corner_tl = "╔";
        let corner_tr = "╗";
        let corner_bl = "╚";
        let corner_br = "╝";
        let side_l = "║";
        let side_r = "║";

        println!("\n{}{}{}", corner_tl, border, corner_tr);
        println!("{}{}{}", side_l, title_border, side_r);
        println!("{}{}{}", side_l, border, side_r);
        
        // Print content with proper indentation
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let padded_line = format!("{:<width$}", line, width = width - 2);
            println!("{}{}{}", side_l, padded_line, side_r);
        }
        
        println!("{}{}{}", side_l, border, side_r);
        println!("{}{}{}\n", corner_bl, border, corner_br);
    }

    /// Prints content as markdown with syntax highlighting
    pub fn print_markdown(&mut self, content: &str, title: &str, _border_style: &str) {
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

        let width = 80;
        let border = "═".repeat(width - 2);
        let title_line = format!(" {} ", title);
        let padding = (width - 4 - title_line.len()) / 2;
        let title_border = format!("{}{}{}", 
            "═".repeat(padding), 
            title_line, 
            "═".repeat(width - 4 - padding - title_line.len())
        );

        let corner_tl = "╔";
        let corner_tr = "╗";
        let corner_bl = "╚";
        let corner_br = "╝";
        let side_l = "║";
        let side_r = "║";

        println!("\n{}{}{}", corner_tl, border, corner_tr);
        println!("{}{}{}", side_l, title_border, side_r);
        println!("{}{}{}", side_l, border, side_r);
        
        // Print content with proper indentation
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let padded_line = format!("{:<width$}", line, width = width - 2);
            println!("{}{}{}", side_l, padded_line, side_r);
        }
        
        println!("{}{}{}", side_l, border, side_r);
        println!("{}{}{}\n", corner_bl, border, corner_br);
    }

    /// Renders markdown text with proper formatting and colors
    pub fn render_markdown(&self, text: &str) {
        let lines: Vec<&str> = text.lines().collect();

        for line in lines {
            let line = line.trim();

            if let Some(header) = line.strip_prefix("### ") {
                // H3 headers - Section headers with underline
                println!("{}", header.bold().cyan());
                println!("{}", "─".repeat(header.len()).cyan());
            } else if let Some(header) = line.strip_prefix("#### ") {
                // H4 headers - Subsection headers with subtle styling
                println!("{}", format!("  {}  ", header).bold().magenta().on_black());
            } else if let Some(header) = line.strip_prefix("## ") {
                // H2 headers - Major section headers with double underline
                println!();
                println!("{}", header.bold().blue());
                println!("{}", "═".repeat(header.len()).blue());
                println!();
            } else if let Some(header) = line.strip_prefix("# ") {
                // H1 headers - Document title with prominent styling
                println!();
                println!("{}", "╔".repeat(header.len() + 4).green());
                println!("║ {} ║", header.bold().green());
                println!("{}", "╚".repeat(header.len() + 4).green());
                println!();
            } else if line.starts_with("**") && line.ends_with("**") {
                // Bold text with enhanced styling
                let bold_text = &line[2..line.len() - 2];
                println!("{}", format!("  {}  ", bold_text).bold().white().on_blue());
            } else if let Some(bullet) = line.strip_prefix("- ") {
                // Bullet points with styled markers
                println!("{}", format!("  • {}  ", bullet).white());
            } else if line.starts_with("---") {
                // Horizontal rules with enhanced styling
                println!("{}", "┄".repeat(80).dimmed());
            } else if line.is_empty() {
                // Empty lines
                println!();
            } else if line.starts_with("```") {
                // Code blocks with enhanced styling
                if line == "```" {
                    println!("{}", "┌".repeat(70).dimmed());
                } else {
                    let lang = &line[3..];
                    println!("{}", format!("┌─ {} ─", lang).dimmed());
                }
            } else if line == "```" {
                // End of code block
                println!("{}", "└".repeat(70).dimmed());
            } else if line.starts_with("`") && line.ends_with("`") {
                // Inline code with styling
                let code = &line[1..line.len()-1];
                println!("{}", format!(" `{}` ", code).white().on_black());
            } else if let Some(quote) = line.strip_prefix("> ") {
                // Blockquotes with enhanced styling
                println!("{}", format!("  │ {}  ", quote).italic().dimmed());
            } else {
                // Regular text with enhanced paragraph styling
                if !line.is_empty() {
                    println!("{}", format!("  {}", line).white());
                }
            }
        }
    }

    /// Renders a section header with consistent styling
    pub fn render_section_header(&self, title: &str) {
        println!("{}", format!("[SECTION] {}", title).bold().cyan());
        println!("{}", "─".repeat(60).dimmed());
    }

    /// Renders a success message
    pub fn render_success(&self, message: &str) {
        println!("{}", format!("[SUCCESS] {}", message).green());
    }

    /// Renders an error message
    pub fn render_error(&self, message: &str) {
        println!("{}", format!("[ERROR] {}", message).red());
    }

    /// Renders a warning message
    pub fn render_warning(&self, message: &str) {
        println!("{}", format!("[WARNING] {}", message).yellow());
    }

    /// Renders an info message
    pub fn render_info(&self, message: &str) {
        println!("{}", format!("[INFO] {}", message).cyan());
    }

    /// Renders workflow completion
    pub fn render_workflow_completion(&self, workflow_name: &str) {
        println!("{}", format!("[COMPLETE] {} finished successfully!", workflow_name).bold().green());
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new(true) // Enable markdown by default
    }
}

// Convenience functions for easy access
pub fn render_markdown(text: &str) {
    let formatter = Formatter::new(true);
    formatter.render_markdown(text);
}

pub fn render_section_header(title: &str) {
    let formatter = Formatter::new(true);
    formatter.render_section_header(title);
}

pub fn render_success(message: &str) {
    let formatter = Formatter::new(true);
    formatter.render_success(message);
}

pub fn render_error(message: &str) {
    let formatter = Formatter::new(true);
    formatter.render_error(message);
}

pub fn render_info(message: &str) {
    let formatter = Formatter::new(true);
    formatter.render_info(message);
}

pub fn render_warning(message: &str) {
    let formatter = Formatter::new(true);
    formatter.render_warning(message);
}

pub fn render_workflow_completion(workflow_name: &str) {
    let formatter = Formatter::new(true);
    formatter.render_workflow_completion(workflow_name);
}

/// Initialize the formatter (for compatibility with existing code)
pub fn init_formatter(_markdown: bool) {
    // No-op since we removed global state management
    // This function exists for API compatibility
}