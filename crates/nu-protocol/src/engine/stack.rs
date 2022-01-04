use std::collections::{HashMap, HashSet};

use crate::engine::EngineState;
use crate::{Config, ShellError, Value, VarId, CONFIG_VARIABLE_ID};

/// A runtime value stack used during evaluation
///
/// A note on implementation:
///
/// We previously set up the stack in a traditional way, where stack frames had parents which would
/// represent other frames that you might return to when exiting a function.
///
/// While experimenting with blocks, we found that we needed to have closure captures of variables
/// seen outside of the blocks, so that they blocks could be run in a way that was both thread-safe
/// and followed the restrictions for closures applied to iterators. The end result left us with
/// closure-captured single stack frames that blocks could see.
///
/// Blocks make up the only scope and stack definition abstraction in Nushell. As a result, we were
/// creating closure captures at any point we wanted to have a Block value we could safely evaluate
/// in any context. This meant that the parents were going largely unused, with captured variables
/// taking their place. The end result is this, where we no longer have separate frames, but instead
/// use the Stack as a way of representing the local and closure-captured state.
#[derive(Debug, Clone)]
pub struct Stack {
    /// Variables
    pub vars: HashMap<VarId, Value>,
    /// Environment variables arranged as a stack to be able to recover values from parent scopes
    pub env_vars: Vec<HashMap<String, Value>>,
    /// Tells which environment variables from engine state are hidden. We don't need to track the
    /// env vars in the stack since we can just delete them.
    pub env_hidden: HashSet<String>,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            vars: HashMap::new(),
            env_vars: vec![],
            env_hidden: HashSet::new(),
        }
    }

    pub fn with_env(&mut self, env_vars: Vec<HashMap<String, Value>>, env_hidden: HashSet<String>) {
        self.env_vars = env_vars;
        self.env_hidden = env_hidden;
    }

    pub fn get_var(&self, var_id: VarId) -> Result<Value, ShellError> {
        if let Some(v) = self.vars.get(&var_id) {
            return Ok(v.clone());
        }

        Err(ShellError::NushellFailed("variable not found".into()))
    }

    pub fn add_var(&mut self, var_id: VarId, value: Value) {
        self.vars.insert(var_id, value);
    }

    pub fn add_env_var(&mut self, var: String, value: Value) {
        // if the env var was hidden, let's activate it again
        self.env_hidden.remove(&var);

        if let Some(scope) = self.env_vars.last_mut() {
            scope.insert(var, value);
        } else {
            self.env_vars.push(HashMap::from([(var, value)]));
        }
    }

    pub fn collect_captures(&self, captures: &[VarId]) -> Stack {
        let mut output = Stack::new();

        for capture in captures {
            // Note: this assumes we have calculated captures correctly and that commands
            // that take in a var decl will manually set this into scope when running the blocks
            if let Ok(value) = self.get_var(*capture) {
                output.vars.insert(*capture, value);
            }
        }

        // FIXME: this is probably slow
        output.env_vars = self.env_vars.clone();
        output.env_vars.push(HashMap::new());

        let config = self
            .get_var(CONFIG_VARIABLE_ID)
            .expect("internal error: config is missing");
        output.vars.insert(CONFIG_VARIABLE_ID, config);

        output
    }

    /// Flatten the env var scope frames into one frame
    pub fn get_env_vars(&self, engine_state: &EngineState) -> HashMap<String, Value> {
        // TODO: Collecting im::HashMap into regular HashMap... maybe we could try im here as well.
        let mut result: HashMap<String, Value> = engine_state
            .env_vars
            .iter()
            .filter(|(k, _)| !self.env_hidden.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for scope in &self.env_vars {
            result.extend(scope.clone());
        }

        result
    }

    pub fn get_env_var(&self, engine_state: &EngineState, name: &str) -> Option<Value> {
        for scope in self.env_vars.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }

        if let Some(val) = engine_state.env_vars.get(name) {
            if self.env_hidden.contains(name) {
                None
            } else {
                Some(val.clone())
            }
        } else {
            None
        }
    }

    pub fn remove_env_var(&mut self, engine_state: &EngineState, name: &str) -> Option<Value> {
        for scope in self.env_vars.iter_mut().rev() {
            if let Some(v) = scope.remove(name) {
                return Some(v);
            }
        }

        if let Some(val) = engine_state.env_vars.get(name) {
            // the environment variable was found in the engine state => mark it as hidden
            self.env_hidden.insert(name.to_string());
            return Some(val.clone());
        }

        None
    }

    pub fn get_config(&self) -> Result<Config, ShellError> {
        let config = self.get_var(CONFIG_VARIABLE_ID);

        match config {
            Ok(config) => config.into_config(),
            Err(e) => {
                println!("Can't find {} in {:?}", CONFIG_VARIABLE_ID, self);
                Err(e)
            }
        }
    }

    pub fn print_stack(&self) {
        println!("vars:");
        for (var, val) in &self.vars {
            println!("  {}: {:?}", var, val);
        }
        for (i, scope) in self.env_vars.iter().rev().enumerate() {
            println!("env vars, scope {} (from the last);", i);
            for (var, val) in scope {
                println!("  {}: {:?}", var, val.clone().debug_value());
            }
        }
    }
}
