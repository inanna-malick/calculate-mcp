use clap::{Parser, Subcommand, ValueEnum};
use compute_mcp::{evaluate, evaluate_batch, Expression};
use serde::Serialize;
use std::io::{self, BufRead};
use std::env;

#[derive(Parser)]
#[command(name = "compute")]
#[command(about = "A command-line arithmetic calculator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Expression to evaluate (if no subcommand is provided)
    expression: Option<String>,
    
    /// Output format (env: COMPUTE_FORMAT)
    #[arg(short, long, value_enum)]
    format: Option<OutputFormat>,
    
    /// Output errors to stderr instead of stdout (human-friendly mode)
    #[arg(long)]
    stderr: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a single arithmetic expression
    Eval {
        /// The expression to evaluate
        expression: String,
        
        /// Output format (env: COMPUTE_FORMAT)
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,
        
        /// Output errors to stderr instead of stdout
        #[arg(long)]
        stderr: bool,
    },
    
    /// Evaluate multiple expressions in batch
    Batch {
        /// Read expressions from stdin (one per line)
        #[arg(short, long)]
        stdin: bool,
        
        /// Expressions to evaluate
        expressions: Vec<String>,
        
        /// Output format (env: COMPUTE_FORMAT)
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,
        
        /// Output errors to stderr instead of stdout
        #[arg(long)]
        stderr: bool,
    },
    
    /// Interactive REPL mode
    Repl {
        /// Show history on exit
        #[arg(short = 'H', long)]
        show_history: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Plain,
    Json,
    Pretty,
}

#[derive(Serialize)]
struct EvalResult {
    expression: String,
    result: Result<f64, String>,
}

#[derive(Serialize)]
struct BatchResult {
    results: Vec<EvalResult>,
    summary: Summary,
}

#[derive(Serialize)]
struct Summary {
    total: usize,
    successful: usize,
    failed: usize,
}

/// Get the output format from CLI arg, env var, or default (json for better LLM integration)
fn get_format(format: Option<OutputFormat>) -> OutputFormat {
    // First check if format was provided via CLI
    if let Some(fmt) = format {
        return fmt;
    }
    
    // Then check environment variable
    if let Ok(env_format) = env::var("COMPUTE_FORMAT") {
        match env_format.to_lowercase().as_str() {
            "plain" => return OutputFormat::Plain,
            "json" => return OutputFormat::Json,
            "pretty" => return OutputFormat::Pretty,
            _ => {} // fall through to default
        }
    }
    
    // Default to JSON for better LLM integration
    OutputFormat::Json
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Eval { expression, format, stderr }) => {
            evaluate_expression(&expression, get_format(format), stderr);
        }
        Some(Commands::Batch { stdin, expressions, format, stderr }) => {
            let expressions = if stdin {
                read_stdin_expressions()
            } else {
                expressions
            };
            evaluate_batch_expressions(&expressions, get_format(format), stderr);
        }
        Some(Commands::Repl { show_history }) => {
            run_repl(show_history);
        }
        None => {
            // If no subcommand but expression provided, evaluate it
            if let Some(expr) = cli.expression {
                evaluate_expression(&expr, get_format(cli.format), cli.stderr);
            } else {
                eprintln!("Error: No expression provided. Use --help for usage information.");
                std::process::exit(1);
            }
        }
    }
}

fn evaluate_expression(expr: &str, format: OutputFormat, use_stderr: bool) {
    let result = evaluate(expr);
    
    match format {
        OutputFormat::Plain => {
            match result {
                Ok(value) => println!("{}", value),
                Err(e) => {
                    if use_stderr {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("Error: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }
        OutputFormat::Json => {
            let eval_result = EvalResult {
                expression: expr.to_string(),
                result: result.map_err(|e| e.to_string()),
            };
            println!("{}", serde_json::to_string(&eval_result).unwrap());
        }
        OutputFormat::Pretty => {
            match result {
                Ok(value) => println!("{} = {}", expr, value),
                Err(e) => {
                    if use_stderr {
                        eprintln!("Error evaluating '{}': {}", expr, e);
                    } else {
                        println!("Error evaluating '{}': {}", expr, e);
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}

fn evaluate_batch_expressions(expressions: &[String], format: OutputFormat, use_stderr: bool) {
    let expr_refs: Vec<Expression> = expressions
        .iter()
        .filter_map(|s| Expression::new(s.clone()))
        .collect();
    
    let results = evaluate_batch(&expr_refs);
    
    let eval_results: Vec<EvalResult> = expressions
        .iter()
        .zip(results.iter())
        .map(|(expr, result)| EvalResult {
            expression: expr.clone(),
            result: result.value.clone().map_err(|e| e.to_string()),
        })
        .collect();
    
    let successful = results.iter().filter(|r| r.value.is_ok()).count();
    let failed = results.len() - successful;
    
    match format {
        OutputFormat::Plain => {
            for (expr, result) in expressions.iter().zip(results.iter()) {
                match &result.value {
                    Ok(value) => println!("{} = {}", expr, value),
                    Err(e) => {
                        if use_stderr {
                            eprintln!("{}: Error: {}", expr, e);
                        } else {
                            println!("{}: Error: {}", expr, e);
                        }
                    }
                }
            }
        }
        OutputFormat::Json => {
            let batch_result = BatchResult {
                results: eval_results,
                summary: Summary {
                    total: expressions.len(),
                    successful,
                    failed,
                },
            };
            println!("{}", serde_json::to_string(&batch_result).unwrap());
        }
        OutputFormat::Pretty => {
            println!("Batch Evaluation Results:");
            println!("========================");
            for (expr, result) in expressions.iter().zip(results.iter()) {
                match &result.value {
                    Ok(value) => println!("✓ {} = {}", expr, value),
                    Err(e) => println!("✗ {}: {}", expr, e),
                }
            }
            println!("------------------------");
            println!("Summary: {} successful, {} failed out of {} total", 
                     successful, failed, expressions.len());
        }
    }
}

fn read_stdin_expressions() -> Vec<String> {
    let stdin = io::stdin();
    stdin.lock()
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .collect()
}

fn run_repl(show_history: bool) {
    let mut history = Vec::new();
    
    println!("Compute REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type expressions to evaluate, 'help' for commands, or 'quit' to exit.");
    println!();
    
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    loop {
        print!("> ");
        io::Write::flush(&mut io::stdout()).unwrap();
        
        let line = match lines.next() {
            Some(Ok(line)) => line,
            _ => break,
        };
        
        let trimmed = line.trim();
        
        match trimmed {
            "quit" | "exit" => break,
            "help" => {
                println!("Commands:");
                println!("  help     - Show this help message");
                println!("  history  - Show calculation history");
                println!("  clear    - Clear history");
                println!("  quit     - Exit REPL");
                println!();
                println!("Examples:");
                println!("  2 + 2");
                println!("  (5 * 3) - 7");
                println!("  3.14159 * 2");
            }
            "history" => {
                if history.is_empty() {
                    println!("No calculations yet.");
                } else {
                    println!("History:");
                    for (i, (expr, result)) in history.iter().enumerate() {
                        match result {
                            Ok(value) => println!("  {}: {} = {}", i + 1, expr, value),
                            Err(e) => println!("  {}: {} (Error: {})", i + 1, expr, e),
                        }
                    }
                }
            }
            "clear" => {
                history.clear();
                println!("History cleared.");
            }
            "" => continue,
            _ => {
                match evaluate(trimmed) {
                    Ok(value) => {
                        println!("{}", value);
                        history.push((trimmed.to_string(), Ok(value)));
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        history.push((trimmed.to_string(), Err(e.to_string())));
                    }
                }
            }
        }
    }
    
    if show_history && !history.is_empty() {
        println!("\nCalculation History:");
        for (expr, result) in &history {
            match result {
                Ok(value) => println!("  {} = {}", expr, value),
                Err(e) => println!("  {} (Error: {})", expr, e),
            }
        }
    }
}