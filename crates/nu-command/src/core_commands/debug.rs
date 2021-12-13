use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Category, Example, PipelineData, ShellError, Signature, Value};

#[derive(Clone)]
pub struct Debug;

impl Command for Debug {
    fn name(&self) -> &str {
        "debug"
    }

    fn usage(&self) -> &str {
        "Debug print the value(s) piped in."
    }

    fn signature(&self) -> Signature {
        Signature::build("debug").category(Category::Core).switch(
            "raw",
            "Prints the raw value representation",
            Some('r'),
        )
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let head = call.head;
        let config = stack.get_config().unwrap_or_default();
        let raw = call.has_flag("raw");

        input.map(
            move |x| {
                if raw {
                    Value::String {
                        val: x.debug_value(),
                        span: head,
                    }
                } else {
                    Value::String {
                        val: x.debug_string(", ", &config),
                        span: head,
                    }
                }
            },
            engine_state.ctrlc.clone(),
        )
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Describe the type of a string",
            example: "'hello' | debug",
            result: Some(Value::test_string("hello")),
        }]
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_examples() {
        use super::Debug;
        use crate::test_examples;
        test_examples(Debug {})
    }
}
