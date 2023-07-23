use std::fmt::Display;

use colored::{ColoredString, Colorize};

use crate::vector::{linspace, Field};

pub fn print_grid<T: Display, F: Fn(Field, Field) -> T>(r: Field, n: usize, f: F) {
    for x in linspace(-r, r, n) {
        let mut line = "".to_string();
        for y in linspace(-r, r, n) {
            line = format!("{} {}", line, f(x, y));
        }
        println!("{}", line);
    }
}

pub fn color_number(n: usize) -> ColoredString {
    match n {
        0 => "0".white(),
        1 => "1".blue(),
        2 => "2".yellow(),
        3 => "3".cyan(),
        4 => "4".green(),
        5 => "5".red(),
        x if x > 9 => format!("{}", x % 10).bright_black(),
        x => format!("{}", x).bright_black(),
    }
}
