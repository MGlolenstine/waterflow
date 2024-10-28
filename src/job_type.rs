use tracing::trace;

use crate::{error::Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WebRequestType {
    #[default]
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum JobType {
    #[default]
    Noop,
    #[cfg(feature = "wasm")]
    Wasm {
        /// Function name to be called with the input
        function_name: String,
        /// Name of the binary that we want to run
        file_name: String,
    },
    Bash {
        /// Command that will be executed inside of Bash
        command: String,
    },
    #[cfg(feature = "web")]
    WebRequest {
        url: String,
        req_type: WebRequestType,
    },
}

impl JobType {
    pub fn new_bash(command: &str) -> Self {
        Self::Bash {
            command: command.to_string(),
        }
    }

    #[cfg(feature = "wasm")]
    pub fn new_wasm(function_name: &str, file_name: &str) -> Self {
        Self::Wasm {
            function_name: function_name.to_string(),
            file_name: file_name.to_string(),
        }
    }

    #[cfg(feature = "web")]
    pub fn new_web_request(url: &str, req_type: WebRequestType) -> Self {
        Self::WebRequest {
            url: url.to_string(),
            req_type,
        }
    }

    pub fn execute(&self, _fixed_inputs: &[String], inputs: &[String]) -> Result<String> {
        match self {
            JobType::Noop => {
                trace!("Noop has been hit!");
                Ok("Noop has been hit!".to_string())
            }
            #[cfg(feature = "wasm")]
            JobType::Wasm {
                function_name,
                file_name,
            } => JobType::execute_wasm(function_name, file_name, inputs),
            JobType::Bash { command } => JobType::execute_bash(command, inputs),
            #[cfg(feature = "web")]
            JobType::WebRequest { url, req_type } => {
                JobType::execute_web_request(url, *req_type, inputs)
            }
        }
    }

    #[cfg(feature = "wasm")]
    fn execute_wasm(function_name: &str, file_name: &str, inputs: &[String]) -> Result<String> {
        use crate::wasm::run_wasm_code;

        run_wasm_code(function_name, file_name, inputs)
    }

    // TODO: implement better kind of placeholding "{INPUT}"
    fn execute_bash(command: &str, inputs: &[String]) -> Result<String> {
        trace!("Inputs: {:?}", inputs);
        let command = command.replace("{INPUT}", &inputs.join(" "));

        let output = std::process::Command::new("bash")
            .args(["-c", &command])
            .output()?;

        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout).to_string();
            trace!("Bash execution succeeded: {}", output);
            Ok(output)
        } else {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            trace!("Bash execution failed: {}", err);
            Err(Error::Bash { e: err })
        }
    }

    #[cfg(feature = "web")]
    fn execute_web_request(
        url: &str,
        req_type: WebRequestType,
        inputs: &[String],
    ) -> Result<String> {
        trace!("Inputs: {:?}", inputs);

        let body = match req_type {
            WebRequestType::Get => ureq::get(url).call()?.into_string()?,
            WebRequestType::Post => ureq::post(url).call()?.into_string()?,
        };

        Ok(body)
    }
}
