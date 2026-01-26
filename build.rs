use std::path::Path;

fn main() {
    let langs = vec!["kotlin", "lua"];
    let root = Path::new("vendor");

    for lang in langs {
        let src_dir = root.join(format!("tree-sitter-{}", lang)).join("src");
        if !src_dir.exists() {
            println!("cargo:warning=Tree-sitter source for {} not found at {:?}", lang, src_dir);
            continue;
        }

        let mut build = cc::Build::new();
        build.include(&src_dir);
        build.file(src_dir.join("parser.c"));

        let scanner_c = src_dir.join("scanner.c");
        let scanner_cc = src_dir.join("scanner.cc");

        if scanner_c.exists() {
            build.file(scanner_c);
        } else if scanner_cc.exists() {
            build.cpp(true); // Enable C++ compilation for scanner.cc
            build.file(scanner_cc);
        }

        build.compile(&format!("tree-sitter-{}", lang));
        println!("cargo:rerun-if-changed={}", src_dir.display());
    }
}
