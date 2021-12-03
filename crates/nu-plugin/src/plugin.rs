use crate::plugin_call::{self, decode_call, encode_response};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command as CommandSys, Stdio};

use nu_protocol::engine::{Command, EngineState, Stack, StateWorkingSet};
use nu_protocol::{ast::Call, Signature, Value};
use nu_protocol::{PipelineData, ShellError};

use super::evaluated_call::EvaluatedCall;

const OUTPUT_BUFFER_SIZE: usize = 8192;

#[derive(Debug)]
pub struct CallInfo {
    pub name: String,
    pub call: EvaluatedCall,
    pub input: Value,
}

// Information sent to the plugin
#[derive(Debug)]
pub enum PluginCall {
    Signature,
    CallInfo(Box<CallInfo>),
}

// Information received from the plugin
#[derive(Debug)]
pub enum PluginResponse {
    Error(String),
    Signature(Vec<Signature>),
    Value(Box<Value>),
}

pub fn get_signature(path: &Path) -> Result<Vec<Signature>, ShellError> {
    let mut plugin_cmd = create_command(path);

    let mut child = plugin_cmd.spawn().map_err(|err| {
        ShellError::PluginFailedToLoad(format!("Error spawning child process: {}", err))
    })?;

    // Create message to plugin to indicate that signature is required and
    // send call to plugin asking for signature
    if let Some(stdin_writer) = &mut child.stdin {
        let mut writer = stdin_writer;
        plugin_call::encode_call(&PluginCall::Signature, &mut writer)?
    }

    // deserialize response from plugin to extract the signature
    let signature = if let Some(stdout_reader) = &mut child.stdout {
        let reader = stdout_reader;
        let mut buf_read = BufReader::with_capacity(OUTPUT_BUFFER_SIZE, reader);
        let response = plugin_call::decode_response(&mut buf_read)?;

        match response {
            PluginResponse::Signature(sign) => Ok(sign),
            PluginResponse::Error(msg) => Err(ShellError::PluginFailedToLoad(format!(
                "Plugin response error {}",
                msg,
            ))),
            _ => Err(ShellError::PluginFailedToLoad(
                "Plugin missing signature".into(),
            )),
        }
    } else {
        Err(ShellError::PluginFailedToLoad(
            "Plugin missing stdout reader".into(),
        ))
    }?;

    // There is no need to wait for the child process to finish since the
    // signature has being collected
    Ok(signature)
}

fn create_command(path: &Path) -> CommandSys {
    //TODO. The selection of shell could be modifiable from the config file.
    let mut process = if cfg!(windows) {
        let mut process = CommandSys::new("cmd");
        process.arg("/c").arg(path);

        process
    } else {
        let mut process = CommandSys::new("sh");
        process.arg("-c").arg(path);

        process
    };

    // Both stdout and stdin are piped so we can receive information from the plugin
    process.stdout(Stdio::piped()).stdin(Stdio::piped());

    process
}

#[derive(Debug, Clone)]
pub struct PluginDeclaration {
    name: String,
    signature: Signature,
    filename: PathBuf,
}

impl PluginDeclaration {
    pub fn new(filename: PathBuf, signature: Signature) -> Self {
        Self {
            name: signature.name.clone(),
            signature,
            filename,
        }
    }
}

impl Command for PluginDeclaration {
    fn name(&self) -> &str {
        &self.name
    }

    fn signature(&self) -> Signature {
        self.signature.clone()
    }

    fn usage(&self) -> &str {
        self.signature.usage.as_str()
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        // Call the command with self path
        // Decode information from plugin
        // Create PipelineData
        let source_file = Path::new(&self.filename);
        let mut plugin_cmd = create_command(source_file);

        let mut child = plugin_cmd.spawn().map_err(|err| {
            let decl = engine_state.get_decl(call.decl_id);
            ShellError::LabeledError(
                format!("Unable to spawn plugin for {}", decl.name()),
                format!("{}", err),
                call.head,
            )
        })?;

        let input = match input {
            PipelineData::Value(value, ..) => value,
            PipelineData::Stream(stream, ..) => {
                let values = stream.collect::<Vec<Value>>();

                Value::List {
                    vals: values,
                    span: call.head,
                }
            }
        };

        // Create message to plugin to indicate that signature is required and
        // send call to plugin asking for signature
        if let Some(stdin_writer) = &mut child.stdin {
            // PluginCall information
            let plugin_call = PluginCall::CallInfo(Box::new(CallInfo {
                name: self.name.clone(),
                call: EvaluatedCall::try_from_call(call, engine_state, stack)?,
                input,
            }));

            let mut writer = stdin_writer;

            plugin_call::encode_call(&plugin_call, &mut writer).map_err(|err| {
                let decl = engine_state.get_decl(call.decl_id);
                ShellError::LabeledError(
                    format!("Unable to encode call for {}", decl.name()),
                    err.to_string(),
                    call.head,
                )
            })?;
        }

        // Deserialize response from plugin to extract the resulting value
        let pipeline_data = if let Some(stdout_reader) = &mut child.stdout {
            let reader = stdout_reader;
            let mut buf_read = BufReader::with_capacity(OUTPUT_BUFFER_SIZE, reader);
            let response = plugin_call::decode_response(&mut buf_read).map_err(|err| {
                let decl = engine_state.get_decl(call.decl_id);
                ShellError::LabeledError(
                    format!("Unable to decode call for {}", decl.name()),
                    err.to_string(),
                    call.head,
                )
            })?;

            match response {
                PluginResponse::Value(value) => {
                    Ok(PipelineData::Value(value.as_ref().clone(), None))
                }
                PluginResponse::Error(msg) => Err(ShellError::LabeledError(
                    "Error received from plugin".into(),
                    msg,
                    call.head,
                )),
                _ => Err(ShellError::LabeledError(
                    "Plugin missing value".into(),
                    "No value received from plugin".into(),
                    call.head,
                )),
            }
        } else {
            Err(ShellError::LabeledError(
                "Error with stdout reader".into(),
                "no stdout reader".into(),
                call.head,
            ))
        }?;

        // There is no need to wait for the child process to finish
        // The response has been collected from the plugin call
        Ok(pipeline_data)
    }

    fn is_plugin(&self) -> Option<&PathBuf> {
        Some(&self.filename)
    }
}

/// The `Plugin` trait defines the API which plugins use to "hook" into nushell.
pub trait Plugin {
    fn signature(&self) -> Vec<Signature>;
    fn run(&mut self, name: &str, call: &EvaluatedCall, input: &Value)
        -> Result<Value, ShellError>;
}

// Function used in the plugin definition for the communication protocol between
// nushell and the external plugin.
// If you want to create a new plugin you have to use this function as the main
// entry point for the plugin
pub fn serve_plugin(plugin: &mut impl Plugin) {
    let mut stdin_buf = BufReader::with_capacity(OUTPUT_BUFFER_SIZE, std::io::stdin());
    let plugin_call = decode_call(&mut stdin_buf);

    match plugin_call {
        Err(err) => {
            let response = PluginResponse::Error(err.to_string());
            encode_response(&response, &mut std::io::stdout()).expect("Error encoding response");
        }
        Ok(plugin_call) => {
            match plugin_call {
                // Sending the signature back to nushell to create the declaration definition
                PluginCall::Signature => {
                    let response = PluginResponse::Signature(plugin.signature());
                    encode_response(&response, &mut std::io::stdout())
                        .expect("Error encoding response");
                }
                PluginCall::CallInfo(call_info) => {
                    let value = plugin.run(&call_info.name, &call_info.call, &call_info.input);

                    let response = match value {
                        Ok(value) => PluginResponse::Value(Box::new(value)),
                        Err(err) => PluginResponse::Error(err.to_string()),
                    };
                    encode_response(&response, &mut std::io::stdout())
                        .expect("Error encoding response");
                }
            }
        }
    }
}

pub fn eval_plugin_signatures(working_set: &mut StateWorkingSet) -> Result<(), ShellError> {
    let decls = working_set
        .get_signatures()
        .map(|(path, signature)| match signature {
            Some(signature) => {
                let plugin_decl = PluginDeclaration::new(path.clone(), signature.clone());
                let plugin_decl: Box<dyn Command> = Box::new(plugin_decl);
                Ok(vec![plugin_decl])
            }
            None => match get_signature(path.as_path()) {
                Ok(signatures) => Ok(signatures
                    .into_iter()
                    .map(|signature| {
                        let plugin_decl = PluginDeclaration::new(path.clone(), signature);
                        let plugin_decl: Box<dyn Command> = Box::new(plugin_decl);
                        plugin_decl
                    })
                    .collect::<Vec<Box<dyn Command>>>()),
                Err(err) => Err(ShellError::PluginFailedToLoad(format!("{}", err))),
            },
        })
        // Need to collect the vector in order to check the error from getting the signature
        .collect::<Result<Vec<Vec<Box<dyn Command>>>, ShellError>>()?;

    for decl in decls.into_iter().flatten() {
        working_set.add_decl(decl);
    }

    Ok(())
}
