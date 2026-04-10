// Octopod Colors - Based on ominky_old theme
pub const INK_PRIMARY: &str = "\x1b[38;5;141m"; // Purple
pub const INK_SECONDARY: &str = "\x1b[38;5;75m"; // Blue
pub const INK_ACCENT: &str = "\x1b[38;5;208m"; // Orange/Coral
pub const INK_SUCCESS: &str = "\x1b[38;5;82m"; // Green
pub const INK_WARNING: &str = "\x1b[38;5;220m"; // Yellow
pub const RESET: &str = "\x1b[0m";
pub const ICON_CHECK: &str = "✓";
pub const ICON_INK: &str = "🐙";

pub const OCTOPUS_ART: &str = r#"                           ___
                        .-'   `'.
                       /         \
                       |         ;
                       |         |           ___.--,
              _.._     |0) ~ (0) |    _.---'`__.-( (_.
       __.--'`_.. '.__.\    '--. \_.-' ,.--'`     `""`
      ( ,.--'`   ',__ /./;   ;, '.__.'`    __
      _`) )  .---.__.' / |   |\   \__..--""  """--.,_
     `---' .'.''-._.-'`_./  /\ '.  \ _.-~~~````~~~-._`-.__.'
           | |  .' _.-' |  |  \  \  '.               `~---`
            \ \/ .'     \  \   '. '-._)
             \/ /        \  \    `=.__`~-.
             / /\         `) )    / / `"".`\
       , _.-'.'\ \        / /    ( (     / /
        `--~`   ) )    .-'.'      '.'.  | (
               (/`    ( (`          ) )  '-;
                `      '-;         (-'"#;

pub fn print_banner() {
    println!("{}{}{}", INK_PRIMARY, OCTOPUS_ART, RESET);
    println!();
    println!(
        "{}Octopod{} - Many-Armed Company Orchestration",
        INK_ACCENT, RESET
    );
    println!();
}

pub fn print_welcome() {
    print_banner();

    println!("Like an octopus coordinating its many arms, Octopod helps you manage");
    println!("AI-powered software development teams across multiple departments.");
    println!();
    println!("You'll act as CEO, spawning and managing agents for:");
    println!();
    println!("  {0}•{1} Product Management", INK_PRIMARY, RESET);
    println!("  {0}•{1} Engineering & Development", INK_PRIMARY, RESET);
    println!("  {0}•{1} Quality Assurance", INK_PRIMARY, RESET);
    println!("  {0}•{1} Design & UX", INK_PRIMARY, RESET);
    println!("  {0}•{1} DevOps & Infrastructure", INK_PRIMARY, RESET);
    println!("  {0}•{1} Marketing", INK_PRIMARY, RESET);
    println!("  {0}•{1} Sales", INK_PRIMARY, RESET);
    println!();
    println!("All working together to build your software products.");
}

pub fn success(msg: &str) {
    println!("{}{} {}{}", INK_SUCCESS, ICON_CHECK, msg, RESET);
}

pub fn error(msg: &str) {
    println!("{}✗ {}{}", INK_WARNING, msg, RESET);
}

pub fn warning(msg: &str) {
    println!("{}⚠ {}{}", INK_WARNING, msg, RESET);
}

pub fn info(msg: &str) {
    println!("{}ℹ {}{}", INK_SECONDARY, msg, RESET);
}

pub fn section(msg: &str) {
    println!("\n{}{}{}", INK_PRIMARY, msg, RESET);
    println!("{}", "─".repeat(msg.len()));
}

pub fn prompt(msg: &str) {
    print!("{}{}: {}{}", INK_ACCENT, msg, RESET, INK_SECONDARY);
}

pub fn prompt_continue() -> anyhow::Result<()> {
    print!("\nPress Enter to continue...");
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut _input = String::new();
    std::io::stdin().read_line(&mut _input)?;
    println!();
    Ok(())
}

#[macro_export]
macro_rules! ink_println {
    ($color:expr, $fmt:literal $(, $arg:expr)*) => {
        println!(concat!("{}", $fmt, "{}"), $color $(, $arg)* , $crate::cli::onboarding::banner::RESET);
    };
}

pub use ink_println;
