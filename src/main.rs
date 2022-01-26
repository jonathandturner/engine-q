mod config_files;
mod eval_file;
mod logger;
mod prompt_update;
mod reedline_config;
mod repl;
mod utils;

#[cfg(test)]
mod tests;

use miette::Result;
use nu_command::{create_default_context, BufferedReader};
use nu_engine::{get_full_help, CallExt};
use nu_parser::parse;
use nu_protocol::{
    ast::{Call, Expr, Expression, Pipeline, Statement},
    engine::{Command, EngineState, Stack, StateWorkingSet},
    ByteStream, Category, IntoPipelineData, PipelineData, ShellError, Signature, Span, Spanned,
    Value,
};
use std::{
    io::BufReader,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use utils::report_error;

fn main() -> Result<()> {
    // miette::set_panic_hook();
    let miette_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |x| {
        crossterm::terminal::disable_raw_mode().expect("unable to disable raw mode");
        miette_hook(x);
    }));

    // Get initial current working directory.
    let init_cwd = utils::get_init_cwd();
    let mut engine_state = create_default_context(&init_cwd);

    // Custom additions
    let delta = {
        let mut working_set = nu_protocol::engine::StateWorkingSet::new(&engine_state);
        working_set.add_decl(Box::new(nu_cli::NuHighlight));

        working_set.render()
    };
    let _ = engine_state.merge_delta(delta, None, &init_cwd);

    // TODO: make this conditional in the future
    // Ctrl-c protection section
    let ctrlc = Arc::new(AtomicBool::new(false));
    let handler_ctrlc = ctrlc.clone();
    let engine_state_ctrlc = ctrlc.clone();

    ctrlc::set_handler(move || {
        handler_ctrlc.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    engine_state.ctrlc = Some(engine_state_ctrlc);
    // End ctrl-c protection section

    let mut args_to_nushell = vec![];
    let mut script_name = String::new();
    let mut args_to_script = vec![];

    // Would be nice if we had a way to parse this. The first flags we see will be going to nushell
    // then it'll be the script name
    // then the args to the script

    let mut collect_arg_nushell = false;
    for arg in std::env::args().skip(1) {
        if !script_name.is_empty() {
            args_to_script.push(arg);
        } else if collect_arg_nushell {
            args_to_nushell.push(arg);
            collect_arg_nushell = false;
        } else if arg.starts_with('-') {
            // Cool, it's a flag
            if arg == "-c"
                || arg == "--commands"
                || arg == "--develop"
                || arg == "--debug"
                || arg == "--loglevel"
                || arg == "--config-file"
            {
                collect_arg_nushell = true;
            }
            args_to_nushell.push(arg);
        } else {
            // Our script file
            script_name = arg;
        }
    }

    args_to_nushell.insert(0, "nu".into());

    let nushell_commandline_args = args_to_nushell.join(" ");

    let nushell_config =
        parse_commandline_args(&nushell_commandline_args, &init_cwd, &mut engine_state);

    match nushell_config {
        Ok(nushell_config) => {
            if !script_name.is_empty() {
                let input = if let Some(redirect_stdin) = &nushell_config.redirect_stdin {
                    let stdin = std::io::stdin();
                    let buf_reader = BufReader::new(stdin);

                    PipelineData::ByteStream(
                        ByteStream {
                            stream: Box::new(BufferedReader::new(buf_reader)),
                            ctrlc: Some(ctrlc),
                        },
                        redirect_stdin.span,
                        None,
                    )
                } else {
                    PipelineData::new(Span::new(0, 0))
                };

                eval_file::evaluate(
                    script_name,
                    &args_to_script,
                    init_cwd,
                    &mut engine_state,
                    input,
                )
            } else {
                repl::evaluate(ctrlc, &mut engine_state)
            }
        }
        Err(_) => std::process::exit(1),
    }
}

fn parse_commandline_args(
    commandline_args: &str,
    init_cwd: &Path,
    engine_state: &mut EngineState,
) -> Result<NushellConfig, ShellError> {
    let (block, delta) = {
        let mut working_set = StateWorkingSet::new(engine_state);
        working_set.add_decl(Box::new(Nu));

        let (output, err) = parse(&mut working_set, None, commandline_args.as_bytes(), false);
        if let Some(err) = err {
            report_error(&working_set, &err);

            std::process::exit(1);
        }

        working_set.hide_decl(b"nu");
        (output, working_set.render())
    };

    let _ = engine_state.merge_delta(delta, None, init_cwd);

    // We should have a successful parse now
    if let Some(Statement::Pipeline(Pipeline { expressions })) = block.stmts.get(0) {
        if let Some(Expression {
            expr: Expr::Call(call),
            ..
        }) = expressions.get(0)
        {
            let redirect_stdin = call.get_named_arg("stdin");

            return Ok(NushellConfig { redirect_stdin });
        }
    }

    // Just give the help and exit if the above fails
    let full_help = get_full_help(
        &Nu.signature(),
        &Nu.examples(),
        engine_state,
        &mut Stack::new(),
    );
    println!("{}", full_help);
    std::process::exit(1);
}

struct NushellConfig {
    redirect_stdin: Option<Spanned<String>>,
}

#[derive(Clone)]
struct Nu;

impl Command for Nu {
    fn name(&self) -> &str {
        "nu"
    }

    fn signature(&self) -> Signature {
        Signature::build("nu")
            .switch("stdin", "redirect the stdin", None)
            .category(Category::System)
    }

    fn usage(&self) -> &str {
        "The nushell language and shell."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        Ok(Value::String {
            val: get_full_help(&Nu.signature(), &Nu.examples(), engine_state, stack),
            span: call.head,
        }
        .into_pipeline_data())
    }
}
