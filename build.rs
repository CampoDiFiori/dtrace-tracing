use std::io::BufWriter;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

fn main() {
    let provider = PathBuf::from_str("provider.d").unwrap();
    let out_dir = PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();

    let wrapper = out_dir.join("wrapper.c");
    generate_wrapper_source_file(&provider, &wrapper).unwrap();

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

    // Generate an object file that encapsulates DTrace probes
    let obj = {
        let res = out_dir.join("wrapper.o");
        let status = cc::Build::new()
            .get_compiler()
            .to_command()
            .arg("-c")
            .arg(wrapper)
            .arg("-o")
            .arg(&res)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(res.exists());
        res
    };

    // Link provider and probe definitions with a user application
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
        let res = out_dir.join("libwrapper.so");
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
        assert!(status.success());
        assert!(res.exists());
    }

    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", out_dir.display());
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=wrapper");

    println!("cargo:rerun-if-changed={}", provider.display());
}

fn generate_wrapper_source_file(
    provider: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = dtrace_parser::File::from_file(provider)?;
    let provider = file.providers().first().cloned().unwrap();

    let provider_name = provider.name.clone();

    let output = std::fs::File::create(output_path)?;
    let mut writer = BufWriter::new(output);
    writeln!(writer, r#"#include "provider.h""#)?;
    writeln!(writer)?;

    for probe in provider.probes {
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
    }

    writer.flush()?;

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
