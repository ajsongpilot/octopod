use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};

pub struct TerminalUI;

impl Default for TerminalUI {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalUI {
    pub fn new() -> Self {
        Self
    }

    pub fn clear_screen(&self) -> Result<()> {
        io::stdout().execute(Clear(ClearType::All))?;
        io::stdout().execute(MoveTo(0, 0))?;
        Ok(())
    }

    pub fn print_header(&self, title: &str) {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║ {:^56} ║", title);
        println!("╚══════════════════════════════════════════════════════════╝\n");
    }

    pub fn print_success(&self, message: &str) {
        println!("  ✓ {}", message);
    }

    pub fn print_error(&self, message: &str) {
        println!("  ✗ {}", message);
    }

    pub fn print_warning(&self, message: &str) {
        println!("  ⚠ {}", message);
    }

    pub fn print_info(&self, message: &str) {
        println!("  ℹ {}", message);
    }

    pub fn print_step(&self, step: u8, total: u8, title: &str) {
        println!("\n  Step {} of {}: {}", step, total, title);
        println!("  {}", "─".repeat(50));
    }

    pub fn prompt(&self, message: &str) -> Result<String> {
        print!("\n{}: ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_string())
    }

    pub fn prompt_yes_no(&self, message: &str, default: bool) -> Result<bool> {
        let default_str = if default { "Y/n" } else { "y/N" };
        print!("\n{} [{}]: ", message, default_str);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();

        if input.is_empty() {
            Ok(default)
        } else {
            Ok(input == "y" || input == "yes")
        }
    }

    pub fn prompt_selection(&self, title: &str, options: &[&str]) -> Result<usize> {
        println!("\n{}", title);
        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, option);
        }

        loop {
            print!("\nSelect [1-{}]: ", options.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice > 0 && choice <= options.len() {
                    return Ok(choice - 1);
                }
            }

            self.print_error("Invalid selection. Please try again.");
        }
    }

    pub fn show_spinner(&self, message: &str) {
        print!("\n  ⠋ {}", message);
        io::stdout().flush().unwrap();
    }

    pub fn clear_spinner(&self) {
        print!("\r  \r");
        io::stdout().flush().unwrap();
    }

    pub fn print_code_block(&self, code: &str) {
        println!("  ```");
        for line in code.lines() {
            println!("  {}", line);
        }
        println!("  ```");
    }

    pub fn wait_for_key(&self) -> Result<()> {
        print!("\nPress Enter to continue...");
        io::stdout().flush()?;

        let mut _input = String::new();
        io::stdin().read_line(&mut _input)?;

        Ok(())
    }

    pub fn print_divider(&self) {
        println!("\n{}", "═".repeat(60));
    }
}
