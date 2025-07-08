use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if protoc is available and try to compile
    if let Ok(protoc_path) = env::var("PROTOC") {
        println!("cargo:warning=Using PROTOC from environment: {}", protoc_path);
        compile_protobuf()?;
    } else if which::which("protoc").is_ok() {
        println!("cargo:warning=Found protoc in PATH, compiling protobuf definitions");
        compile_protobuf()?;
    } else {
        println!("cargo:warning=protoc not found in PATH");
        println!("cargo:warning=Using fallback protobuf implementations");
        println!("cargo:warning=To enable full protobuf support, install protoc or set PROTOC environment variable");
        println!("cargo:warning=Project will compile and work with manual implementations");
    }

    Ok(())
}

fn compile_protobuf() -> Result<(), Box<dyn std::error::Error>> {
    let result = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["raft.proto"], &["."]);

    match result {
        Ok(_) => {
            println!("cargo:rustc-cfg=feature=\"generated-proto\"");
            println!("cargo:warning=Successfully compiled protobuf definitions");
        },
        Err(e) => {
            println!("cargo:warning=Failed to compile protobuf: {}", e);
            println!("cargo:warning=Using fallback implementations");
        }
    }

    Ok(())
}
