mod declaration;
pub use declaration::PluginDeclaration;

use crate::protocol::{LabeledError, PluginCall, PluginResponse};
use crate::EncodingType;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command as CommandSys, Stdio};

use nu_protocol::ShellError;
use nu_protocol::{Signature, Value};

use super::EvaluatedCall;

const OUTPUT_BUFFER_SIZE: usize = 8192;

pub trait PluginEncoder: Clone {
    fn encode_call(
        &self,
        plugin_call: &PluginCall,
        writer: &mut impl std::io::Write,
    ) -> Result<(), ShellError>;

    fn decode_call(&self, reader: &mut impl std::io::BufRead) -> Result<PluginCall, ShellError>;

    fn encode_response(
        &self,
        plugin_response: &PluginResponse,
        writer: &mut impl std::io::Write,
    ) -> Result<(), ShellError>;

    fn decode_response(
        &self,
        reader: &mut impl std::io::BufRead,
    ) -> Result<PluginResponse, ShellError>;
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

pub fn get_signature(path: &Path, encoding: &EncodingType) -> Result<Vec<Signature>, ShellError> {
    eprintln!("1");
    let mut plugin_cmd = create_command(path);

    eprintln!("2");
    let mut child = plugin_cmd.spawn().map_err(|err| {
        ShellError::PluginFailedToLoad(format!("Error spawning child process: {}", err))
    })?;

    eprintln!("3");
    // Create message to plugin to indicate that signature is required and
    // send call to plugin asking for signature
    if let Some(stdin_writer) = &mut child.stdin {
        eprintln!("3.1");
        let mut writer = stdin_writer;
        eprintln!("3.2");
        encoding.encode_call(&PluginCall::Signature, &mut writer)?;
        eprintln!("3.3");
    }

    eprintln!("4");
    // deserialize response from plugin to extract the signature
    let signature = if let Some(stdout_reader) = &mut child.stdout {
        eprintln!("5");
        let reader = stdout_reader;
        eprintln!("6");
        let mut buf_read = BufReader::with_capacity(OUTPUT_BUFFER_SIZE, reader);
        eprintln!("7");
        let response = encoding.decode_response(&mut buf_read)?;

        eprintln!("8");
        match response {
            PluginResponse::Signature(sign) => Ok(sign),
            PluginResponse::Error(err) => Err(err.into()),
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

// The next trait and functions are part of the plugin that is being created
// The `Plugin` trait defines the API which plugins use to "hook" into nushell.
pub trait Plugin {
    fn signature(&self) -> Vec<Signature>;
    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError>;
}

// Function used in the plugin definition for the communication protocol between
// nushell and the external plugin.
// When creating a new plugin you have to use this function as the main
// entry point for the plugin, e.g.
//
// fn main() {
//    serve_plugin(plugin)
// }
//
// where plugin is your struct that implements the Plugin trait
//
// Note. When defining a plugin in other language but Rust, you will have to compile
// the plugin.capnp schema to create the object definitions that will be returned from
// the plugin.
// The object that is expected to be received by nushell is the PluginResponse struct.
// That should be encoded correctly and sent to StdOut for nushell to decode and
// and present its result
pub fn serve_plugin(plugin: &mut impl Plugin, encoder: impl PluginEncoder) {
    eprintln!("in serve_plugin 1");
    let mut stdin_buf = BufReader::with_capacity(OUTPUT_BUFFER_SIZE, std::io::stdin());
    eprintln!("in serve_plugin 2");
    let plugin_call = encoder.decode_call(&mut stdin_buf);
    eprintln!("in serve_plugin 3");

    match plugin_call {
        Err(err) => {
            let response = PluginResponse::Error(err.into());
            encoder
                .encode_response(&response, &mut std::io::stdout())
                .expect("Error encoding response");
        }
        Ok(plugin_call) => {
            match plugin_call {
                // Sending the signature back to nushell to create the declaration definition
                PluginCall::Signature => {
                    eprintln!("signature 1");
                    let response = PluginResponse::Signature(plugin.signature());
                    eprintln!("signature 2");
                    encoder
                        .encode_response(&response, &mut std::io::stdout())
                        .expect("Error encoding response");
                }
                PluginCall::CallInfo(call_info) => {
                    let value = plugin.run(&call_info.name, &call_info.call, &call_info.input);

                    let response = match value {
                        Ok(value) => PluginResponse::Value(Box::new(value)),
                        Err(err) => PluginResponse::Error(err),
                    };
                    encoder
                        .encode_response(&response, &mut std::io::stdout())
                        .expect("Error encoding response");
                }
            }
        }
    }
}
