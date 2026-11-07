#[cfg(feature = "native-onnx")]
use ort::environment::Environment;
#[cfg(feature = "native-onnx")]
use ort::session::Session;
#[cfg(feature = "native-onnx")]
use std::path::Path;
#[cfg(feature = "native-onnx")]
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRequest {
    Cpu,
    DirectMl,
}

impl ProviderRequest {
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            None | Some("cpu") => Ok(Self::Cpu),
            Some("directml") => Ok(Self::DirectMl),
            Some(value) => Err(format!(
                "unsupported ONNX provider '{value}'; expected 'cpu' or 'directml'"
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelection {
    pub requested: ProviderRequest,
    pub active: ProviderRequest,
    pub fallback_reason: Option<String>,
}

#[cfg(feature = "native-onnx")]
static CPU_ENVIRONMENT: OnceLock<Result<Environment, String>> = OnceLock::new();

#[cfg(feature = "native-onnx")]
pub fn cpu_environment() -> Result<Environment, String> {
    CPU_ENVIRONMENT
        .get_or_init(|| {
            ort::init()
                .with_name("hibiki-cpu-analysis")
                .build()
                .map_err(|error| error.to_string())
        })
        .clone()
}

#[cfg(feature = "native-onnx")]
pub fn build_session(
    model_path: &Path,
    request: ProviderRequest,
) -> Result<(Environment, Session, ProviderSelection), String> {
    let environment = cpu_environment()?;
    if request == ProviderRequest::DirectMl {
        match build_directml_session(&environment, model_path) {
            Ok(session) => {
                return Ok((
                    environment,
                    session,
                    ProviderSelection {
                        requested: request,
                        active: ProviderRequest::DirectMl,
                        fallback_reason: None,
                    },
                ));
            }
            Err(reason) => {
                let session = build_cpu_session(&environment, model_path)?;
                return Ok((
                    environment,
                    session,
                    ProviderSelection {
                        requested: request,
                        active: ProviderRequest::Cpu,
                        fallback_reason: Some(reason),
                    },
                ));
            }
        }
    }

    let session = build_cpu_session(&environment, model_path)?;
    Ok((
        environment,
        session,
        ProviderSelection {
            requested: request,
            active: ProviderRequest::Cpu,
            fallback_reason: None,
        },
    ))
}

#[cfg(feature = "native-onnx")]
fn build_cpu_session(environment: &Environment, model_path: &Path) -> Result<Session, String> {
    Session::builder(environment)
        .map_err(|error| error.to_string())?
        .commit_from_file(model_path)
        .map_err(|error| error.to_string())
}

#[cfg(all(windows, feature = "native-onnx-directml"))]
fn build_directml_session(environment: &Environment, model_path: &Path) -> Result<Session, String> {
    use ort::{
        ep::{directml::PerformancePreference, DirectML},
        session::Session,
    };

    Session::builder(environment)
        .map_err(|error| error.to_string())?
        .with_memory_pattern(false)
        .map_err(|error| error.to_string())?
        .with_parallel_execution(false)
        .map_err(|error| error.to_string())?
        .with_execution_providers([DirectML::default()
            .with_performance_preference(PerformancePreference::HighPerformance)
            .build()
            .error_on_failure()])
        .map_err(|error| error.to_string())?
        .commit_from_file(model_path)
        .map_err(|error| error.to_string())
}

#[cfg(not(all(windows, feature = "native-onnx-directml")))]
#[cfg(feature = "native-onnx")]
fn build_directml_session(
    _environment: &Environment,
    _model_path: &Path,
) -> Result<Session, String> {
    Err("DirectML is not enabled in this build; using the CPU fallback".to_owned())
}

#[cfg(all(test, feature = "native-onnx"))]
mod tests {
    use super::{cpu_environment, ProviderRequest};

    #[test]
    fn reuses_one_process_environment_for_all_native_models() {
        let first = cpu_environment().expect("first environment should initialize");
        let second = cpu_environment().expect("second caller should reuse the environment");
        drop((first, second));
    }

    #[test]
    fn parses_only_explicit_development_provider_names() {
        assert_eq!(ProviderRequest::parse(None).unwrap(), ProviderRequest::Cpu);
        assert_eq!(
            ProviderRequest::parse(Some("cpu")).unwrap(),
            ProviderRequest::Cpu
        );
        assert_eq!(
            ProviderRequest::parse(Some("directml")).unwrap(),
            ProviderRequest::DirectMl
        );
        assert!(ProviderRequest::parse(Some("cuda")).is_err());
    }
}

#[cfg(all(test, not(feature = "native-onnx")))]
mod provider_request_tests {
    use super::ProviderRequest;

    #[test]
    fn parses_only_approved_provider_names_without_loading_a_runtime() {
        assert_eq!(ProviderRequest::parse(None).unwrap(), ProviderRequest::Cpu);
        assert_eq!(
            ProviderRequest::parse(Some("cpu")).unwrap(),
            ProviderRequest::Cpu
        );
        assert_eq!(
            ProviderRequest::parse(Some("directml")).unwrap(),
            ProviderRequest::DirectMl
        );
        assert!(ProviderRequest::parse(Some("cuda")).is_err());
    }
}
