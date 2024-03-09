use std::io::BufWriter;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use bindgen::RegexSet;

fn main() {
    let provider = PathBuf::from_str("provider.d").unwrap();
    let out_dir = PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();

    // Run DTrace to generate the .h file from provider.d
    let _h = {
        let res = out_dir.join("provider.h");
        let status = Command::new("dtrace")
            .arg("-h")
            .arg("-s")
            .arg(&provider)
            .arg("-o")
            .arg(&res)
            .status()
            .expect("Failed to generate header file with DTrace");
        assert!(status.success());
        assert!(res.exists());
        res
    };

    let wrapper = out_dir.join("rustracing.c");
    let bindings = PathBuf::from_str("src/bindings.rs").unwrap();
    generate_wrapper_source_file(&provider, &wrapper, &bindings).unwrap();

    // Generate an object file that encapsulates DTrace probes
    let obj = {
        let res = out_dir.join("rustracing.o");
        let status = cc::Build::new()
            .get_compiler()
            .to_command()
            .arg("-c")
            .arg(&wrapper)
            .arg("-o")
            .arg(&res)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(res.exists());
        res
    };

    // Link provider and probe definitions with a user application
    #[cfg(not(target_os = "macos"))]
    let dobj = {
        let res = out_dir.join("provider.o");
        let status = Command::new("dtrace")
            .arg("-G")
            .arg("-s")
            .arg(&provider)
            .arg(&obj)
            .arg("-o")
            .arg(&res)
            .status()
            .expect("Failed to generate object file with DTrace");
        assert!(status.success());
        assert!(res.exists());
        res
    };

    // Compile a .so file that will be linked to the final binary
    {
        let res = out_dir.join("librustracing.so");
        #[cfg(not(target_os = "macos"))]
        let status = cc::Build::new()
            .get_compiler()
            .to_command()
            .arg("-shared")
            .arg("-o")
            .arg(&res)
            .arg("-fPIC")
            .arg(obj)
            .arg(dobj)
            .status()
            .unwrap();
        #[cfg(target_os = "macos")]
        let status = cc::Build::new()
            .get_compiler()
            .to_command()
            .arg("-shared")
            .arg("-o")
            .arg(&res)
            .arg("-fPIC")
            .arg(obj)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(res.exists());
    }

    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", out_dir.display());
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=rustracing");

    println!("cargo:rerun-if-changed={}", provider.display());
}

fn generate_wrapper_source_file(
    provider: &Path,
    output_path: &Path,
    rust_bindings_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = dtrace_parser::File::from_file(provider)?;
    let output = std::fs::File::create(output_path)?;
    let mut writer = BufWriter::new(output);
    writeln!(writer, r#"#include "provider.h""#)?;
    writeln!(writer)?;

    let mut provider_regex = RegexSet::new();

    for provider in file.providers() {
        let provider_name = &provider.name;
        for probe in provider.probes.iter() {
            // probe implementation
            writeln!(
                writer,
                r#"void {}_{}({}) {{"#,
                &provider_name,
                &probe.name,
                collect_args(&probe.types, CollectArgsOptions { with_types: true })
            )?;
            writeln!(
                writer,
                r#"    {}_{}({});"#,
                provider_name.to_uppercase(),
                probe.name.to_uppercase(),
                collect_args(&probe.types, CollectArgsOptions { with_types: false })
            )?;
            writeln!(writer, "}}\n")?;
            // probe enabled
            writeln!(
                writer,
                r#"int {}_{}_enabled(void) {{"#,
                &provider_name, &probe.name,
            )?;
            writeln!(
                writer,
                r#"    return {}_{}_ENABLED();"#,
                provider_name.to_uppercase(),
                probe.name.to_uppercase(),
            )?;
            writeln!(writer, "}}\n")?;
        }

        provider_regex.insert(format!("{provider_name}.*"));
    }

    writer.flush()?;

    bindgen::Builder::default()
        .allowlist_item(provider_regex.get_items().join("|"))
        .header(output_path.to_str().unwrap())
        .generate()
        .unwrap()
        .write_to_file(rust_bindings_path)
        .unwrap();

    Ok(())
}

struct CollectArgsOptions {
    with_types: bool,
}

fn collect_args(args: &[dtrace_parser::DataType], options: CollectArgsOptions) -> String {
    let mut tmp = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        if options.with_types {
            let t = arg.to_c_type();
            tmp.push(format!("{t} arg{i}"));
        } else {
            tmp.push(format!("arg{i}"));
        }
    }

    tmp.join(",")
}
