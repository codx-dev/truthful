use cli_table::{
    format::{Border, Justify, Separator},
    Cell, Style, Table as _,
};
use rustyline::{error::ReadlineError, Config as RustylineConfig, Editor};
use serde::{Deserialize, Serialize};
use std::fs;
use toml_base_config::BaseConfig;
use truthful::*;

/// Readline configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readline {
    max_history_size: usize,
    history_ignore_space: bool,
    auto_add_history: bool,
}

impl From<Readline> for RustylineConfig {
    fn from(config: Readline) -> Self {
        let Readline {
            max_history_size,
            history_ignore_space,
            auto_add_history,
        } = config;

        RustylineConfig::builder()
            .max_history_size(max_history_size)
            .history_ignore_space(history_ignore_space)
            .auto_add_history(auto_add_history)
            .build()
    }
}

impl Default for Readline {
    fn default() -> Self {
        let base = RustylineConfig::default();

        let max_history_size = 500;
        let history_ignore_space = base.history_ignore_space();
        let auto_add_history = true;

        Self {
            max_history_size,
            history_ignore_space,
            auto_add_history,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Options {
    pub cheesy_mode: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { cheesy_mode: true }
    }
}

/// App configuration
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub options: Options,
    pub readline: Readline,
}

impl Config {
    /// Create an instance of the rustyline configuration
    pub fn rustyline(&self) -> RustylineConfig {
        self.readline.into()
    }
}

impl BaseConfig for Config {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
}

fn print_help() {
    println!("enter a logical expression to evaluate. Example: !a v b");
    println!("?, h or help for this list");
    println!("q or quit to exit");
}

fn main() {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("failed to load config: {e}");
        eprintln!("falling back to default config");
        Config::default()
    });

    let rline_config = config.rustyline();
    let mut rl = Editor::<()>::with_config(rline_config).expect("failed to lock stdin rustyline");

    let history = match dirs::data_local_dir()
        .map(|p| p.join(env!("CARGO_PKG_NAME")))
        .map(|p| fs::create_dir_all(&p).map(|_| p.join("history")))
        .transpose()
    {
        Ok(h) => h,
        Err(e) => {
            eprintln!("error reading commands history: {e}");
            None
        }
    };

    if let Some(h) = &history {
        if !h.exists() {
            fs::OpenOptions::new().create_new(true).open(h).ok();
        }

        if h.exists() {
            if let Err(e) = rl.load_history(h) {
                eprintln!("failed to load commands history: {e}");
            }
        }
    }

    println!("welcome! enter ? for help");

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                match line.as_str().to_lowercase().to_string().as_str() {
                    "q" | "quit" => {
                        break;
                    }
                    "?" | "h" | "help" => {
                        print_help();
                        continue;
                    }
                    "" => {
                        continue;
                    }
                    _ => (),
                }

                let instruction = match Instruction::try_from(line.as_str()) {
                    Ok(i) => i,
                    Err(e) => {
                        eprintln!("error parsing line: {e}");
                        continue;
                    }
                };

                if instruction.eq_true() {
                    if config.options.cheesy_mode {
                        println!("how very wet this water is...");
                    } else {
                        println!("true");
                    }
                    continue;
                } else if instruction.eq_false() {
                    if config.options.cheesy_mode {
                        println!("well, that's just, like, your opinion, man...");
                    } else {
                        println!("false");
                    }
                    continue;
                }

                println!("evaluating {instruction}");

                let Table { header, rows } = match instruction.evaluate() {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("error evaluating instruction {instruction}: {e}");
                        continue;
                    }
                };

                let table = rows
                    .iter()
                    .map(|v| {
                        v.iter().map(|v| {
                            v.then(|| "1".cell().bold(true).justify(Justify::Center))
                                .unwrap_or_else(|| "0".cell().justify(Justify::Center))
                        })
                    })
                    .collect::<Vec<_>>()
                    .table()
                    .border(Border::builder().build())
                    .separator(Separator::builder().row(None).build())
                    .title(header);

                let table = match table.display() {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("error displaying the evaluation: {e}");
                        continue;
                    }
                };

                print!("{table}");
            }
            Err(ReadlineError::Interrupted) => {
                //eprintln!("CTRL-C");
            }
            Err(ReadlineError::Eof) => {
                eprintln!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("error reading command: {err}");
            }
        }
    }

    if let Some(h) = history {
        if let Err(e) = rl.save_history(&h) {
            eprintln!("failed to save commands history: {e}");
        }
    }

    println!("bye!");
}
