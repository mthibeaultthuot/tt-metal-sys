use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

fn add_include(build: &mut cc::Build, seen: &mut HashSet<String>, path: &std::path::Path) {
    if path.exists() {
        let s = path.to_string_lossy().to_string();
        if seen.insert(s) {
            build.include(path);
        }
    }
}

fn main() {
    let tt_metal_home = env::var("TT_METAL_HOME").unwrap_or_else(|_| {
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        manifest
            .parent()
            .unwrap()
            .join("tt-metal")
            .to_string_lossy()
            .into_owned()
    });

    let tt_metal_home = PathBuf::from(&tt_metal_home);

    let lib_dir = env::var("TT_METAL_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| tt_metal_home.join("build").join("tt_metal"));

    let mut build = cc::Build::new();
    let mut seen = HashSet::new();

    build
        .cpp(true)
        .std("c++20")
        .file("cpp/wrapper.cpp")
        .include("cpp");

    add_include(
        &mut build,
        &mut seen,
        &tt_metal_home.join("tt_metal").join("api"),
    );
    add_include(
        &mut build,
        &mut seen,
        &tt_metal_home
            .join("tt_metal")
            .join("hostdevcommon")
            .join("api"),
    );
    add_include(&mut build, &mut seen, &tt_metal_home.join("tt_metal"));
    add_include(&mut build, &mut seen, &tt_metal_home);
    add_include(&mut build, &mut seen, &tt_metal_home.join("tt_stl"));
    add_include(
        &mut build,
        &mut seen,
        &tt_metal_home.join("build").join("include"),
    );

    let compile_commands = tt_metal_home.join("build").join("compile_commands.json");
    if compile_commands.exists() {
        if let Ok(contents) = std::fs::read_to_string(&compile_commands) {
            for cap in regex_lite::Regex::new(r"-(?:I|isystem)\s*(\S+)")
                .unwrap()
                .captures_iter(&contents)
            {
                let path = cap.get(1).unwrap().as_str();
                add_include(&mut build, &mut seen, &PathBuf::from(path));
            }
        }
    }

    let cpm_cache = tt_metal_home.join(".cpmcache");
    if cpm_cache.exists() {
        if let Ok(entries) = std::fs::read_dir(&cpm_cache) {
            for entry in entries.flatten() {
                let pkg_dir = entry.path();
                if pkg_dir.is_dir() {
                    if let Ok(hashes) = std::fs::read_dir(&pkg_dir) {
                        for hash_entry in hashes.flatten() {
                            let hash_dir = hash_entry.path();
                            if hash_dir.is_dir() {
                                add_include(&mut build, &mut seen, &hash_dir.join("include"));
                                add_include(&mut build, &mut seen, &hash_dir);
                                // Nested subdirectories (e.g. enchantum/<hash>/enchantum/include/)
                                if let Ok(subdirs) = std::fs::read_dir(&hash_dir) {
                                    for subdir in subdirs.flatten() {
                                        let sub_path = subdir.path();
                                        if sub_path.is_dir() {
                                            let sub_include = sub_path.join("include");
                                            add_include(&mut build, &mut seen, &sub_include);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    add_include(
        &mut build,
        &mut seen,
        &tt_metal_home.join("tt_metal/third_party/umd/device/api"),
    );
    add_include(
        &mut build,
        &mut seen,
        &tt_metal_home.join("tt_metal/third_party/umd/device"),
    );
    add_include(
        &mut build,
        &mut seen,
        &tt_metal_home.join("tt_metal/third_party/umd/device/common"),
    );
    add_include(
        &mut build,
        &mut seen,
        &tt_metal_home.join("build/tt_metal/third_party/umd/device"),
    );

    if let Ok(arch) = env::var("ARCH_NAME") {
        build.define("ARCH_NAME", arch.as_str());
    }

    build.warnings(false).compile("tt_metal_wrapper");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!(
        "cargo:rustc-link-search=native={}",
        tt_metal_home.join("build").join("lib").display()
    );
    println!("cargo:rustc-link-lib=dylib=tt_metal");
    println!("cargo:rustc-link-lib=dylib=device");
    println!("cargo:rustc-link-lib=dylib=fmt");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    println!("cargo:rerun-if-changed=cpp/wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp/wrapper.hpp");
    println!("cargo:rerun-if-env-changed=TT_METAL_HOME");
    println!("cargo:rerun-if-env-changed=TT_METAL_LIB_DIR");
}
