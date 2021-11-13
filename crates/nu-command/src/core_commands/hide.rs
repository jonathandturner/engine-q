use nu_protocol::ast::{Call, Expr, Expression, ImportPatternMember};
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{PipelineData, ShellError, Signature, SyntaxShape};

#[derive(Clone)]
pub struct Hide;

impl Command for Hide {
    fn name(&self) -> &str {
        "hide"
    }

    fn usage(&self) -> &str {
        "Hide definitions in the current scope"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("hide").required("pattern", SyntaxShape::String, "import pattern")
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let import_pattern = if let Some(Expression {
            expr: Expr::ImportPattern(pat),
            ..
        }) = call.positional.get(0)
        {
            pat
        } else {
            return Err(ShellError::InternalError(
                "Got something else than import pattern".into(),
            ));
        };

        let head_name_str = if let Ok(s) = String::from_utf8(import_pattern.head.name.clone()) {
            s
        } else {
            return Err(ShellError::NonUtf8(import_pattern.head.span));
        };

        if let Some(block_id) = engine_state.find_module(&import_pattern.head.name) {
            // The first word is a module
            let overlay = &engine_state.get_block(block_id).overlay;

            let env_vars_to_hide = if import_pattern.members.is_empty() {
                overlay.env_vars_with_head(&import_pattern.head.name)
            } else {
                match &import_pattern.members[0] {
                    ImportPatternMember::Glob { .. } => {
                        overlay.env_vars_with_head(&import_pattern.head.name)
                    }
                    ImportPatternMember::Name { name, span } => {
                        let mut output = vec![];

                        if let Some((name, id)) =
                            overlay.env_var_with_head(name, &import_pattern.head.name)
                        {
                            output.push((name, id));
                        } else if !overlay.has_decl(name) {
                            return Err(ShellError::EnvVarNotFoundAtRuntime(*span));
                        }

                        output
                    }
                    ImportPatternMember::List { names } => {
                        let mut output = vec![];

                        for (name, span) in names {
                            if let Some((name, id)) =
                                overlay.env_var_with_head(name, &import_pattern.head.name)
                            {
                                output.push((name, id));
                            } else if !overlay.has_decl(name) {
                                return Err(ShellError::EnvVarNotFoundAtRuntime(*span));
                            }
                        }

                        output
                    }
                }
            };

            for (name, _) in env_vars_to_hide {
                let name = if let Ok(s) = String::from_utf8(name.clone()) {
                    s
                } else {
                    // TODO: Fix this span
                    return Err(ShellError::NonUtf8(import_pattern.head.span));
                };

                stack.remove_env_var(&name);
            }
        } else if stack.remove_env_var(&head_name_str).is_none()
            && !import_pattern.hidden.contains(&import_pattern.head.name)
        {
            return Err(ShellError::NotFound(call.positional[0].span));
        }

        Ok(PipelineData::new(call.head))
    }
}
