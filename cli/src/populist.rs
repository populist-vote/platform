use colored::*;

static POPULIST_1: &'static str = r#"oooooooooo    ooooooo  oooooooooo ooooo  oooo ooooo       ooooo  oooooooo8 ooooooooooo"#;
static POPULIST_2: &'static str = r#" 888    888 o888   888o 888    888 888    88   888         888  888        88  888  88"#;
static POPULIST_3: &'static str = r#" 888oooo88  888     888 888oooo88  888    88   888         888   888oooooo     888    "#;
static POPULIST_4: &'static str = r#" 888        888o   o888 888        888    88   888      o  888          888    888    "#;
static POPULIST_5: &'static str = r#"o888o         88ooo88  o888o        888oo88   o888ooooo88 o888o o88oooo888    o888o   "#;

pub fn headline() {
    eprintln!("\n{}", POPULIST_1.bright_red());
    eprintln!("{}", POPULIST_2.white());
    eprintln!("{}", POPULIST_3.blue());
    eprintln!("{}", POPULIST_4.bright_red());
    eprintln!("{}", POPULIST_5.white());
}