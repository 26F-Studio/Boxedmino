use slint_build::CompilerConfiguration;

fn main() {
    let config = CompilerConfiguration::new()
        .with_style("cosmic-dark".into());

    slint_build::compile_with_config("ui/main.slint", config)
        .expect("Slint build failed");
}