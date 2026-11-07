#[cfg(feature = "native-onnx")]
use ort::environment::Environment;
#[cfg(feature = "native-onnx")]
use std::sync::OnceLock;

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

#[cfg(all(test, feature = "native-onnx"))]
mod tests {
    use super::cpu_environment;

    #[test]
    fn reuses_one_process_environment_for_all_native_models() {
        let first = cpu_environment().expect("first environment should initialize");
        let second = cpu_environment().expect("second caller should reuse the environment");
        drop((first, second));
    }
}
