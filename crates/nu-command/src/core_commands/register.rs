use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Category, PipelineData, Signature, SyntaxShape};

#[derive(Clone)]
pub struct Register;

impl Command for Register {
    fn name(&self) -> &str {
        "register"
    }

    fn usage(&self) -> &str {
        "Register a plugin"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("register")
            .required(
                "plugin",
                SyntaxShape::Filepath,
                "path of executable for plugin",
            )
            .required_named(
                "encoding",
                SyntaxShape::String,
                "Encoding used to communicate with plugin. Options: [capnp, json]",
                Some('e'),
            )
            .optional(
                "signature",
                SyntaxShape::Any,
                "Block with signature description as json object",
            )
            .named(
                "shell",
                SyntaxShape::Filepath,
                "path of shell used to run plugin (cmd, sh, python, etc)",
                Some('s'),
            )
            .category(Category::Core)
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        Ok(PipelineData::new(call.head))
    }
}
