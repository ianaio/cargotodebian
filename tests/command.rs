use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
#[cfg(all(feature = "lzma", target_os = "linux"))]
fn run_cargo_deb_command_on_example_dir() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let cmd_path = root.join("target/debug/cargotodebian");
    assert!(cmd_path.exists());
    let output = Command::new(cmd_path)
        .arg(format!("--manifest-path={}", root.join("example/Cargo.toml").display()))
        .output().unwrap();
    if !output.status.success() {
        panic!("Cmd failed: {}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    }

    // prints deb path on the last line
    let last_line = output.stdout[..output.stdout.len() - 1].split(|&c| c == b'\n').last().unwrap();
    let deb_path = Path::new(::std::str::from_utf8(last_line).unwrap());
    assert!(deb_path.exists());

    let ardir = tempfile::tempdir().expect("testdir");
    assert!(ardir.path().exists());
    assert!(Command::new("ar")
        .current_dir(ardir.path())
        .arg("-x")
        .arg(deb_path)
        .status().unwrap().success());

    assert_eq!("2.0\n", fs::read_to_string(ardir.path().join("debian-binary")).unwrap());
    assert!(ardir.path().join("data.tar.xz").exists());
    assert!(ardir.path().join("control.tar.xz").exists());

    let cdir = tempfile::tempdir().unwrap();
    assert!(Command::new("tar")
        .arg("xf")
        .current_dir(cdir.path())
        .arg(ardir.path().join("control.tar.xz"))
        .status().unwrap().success());

    let control = fs::read_to_string(cdir.path().join("control")).unwrap();
    assert!(control.contains("Package: example\n"));
    assert!(control.contains("Version: 0.1.0\n"));
    assert!(control.contains("Section: utils\n"));
    assert!(control.contains("Architecture: "));
    assert!(control.contains("Maintainer: cargotodebian developers <cargotodebian@example.invalid>\n"));

    let md5sums = fs::read_to_string(cdir.path().join("md5sums")).unwrap();
    assert!(md5sums.contains(" usr/bin/example\n"));
    assert!(md5sums.contains(" usr/share/doc/example/changelog.Debian.gz\n"));
    assert!(md5sums.contains("83453jk345jk234jk234jk234jk2349s  var/lib/example/1.txt\n"));
    assert!(md5sums.contains("25j8s8sdf8g4891dsf0336m7w654m544  var/lib/example/2.txt\n"));
    assert!(md5sums.contains("345k35a3459ag8f8d9s74k3k33439sdd  var/lib/example/3.txt\n"));
    assert!(md5sums.contains("9l0l2l53hk34jk3trg721kggfd944mm4  usr/share/doc/example/copyright\n"));

    let ddir = tempfile::tempdir().unwrap();
    assert!(Command::new("tar")
        .arg("xJf")
        .current_dir(ddir.path())
        .arg(ardir.path().join("data.tar.xz"))
        .status().unwrap().success());

    assert!(ddir.path().join("var/lib/example/1.txt").exists());
    assert!(ddir.path().join("var/lib/example/2.txt").exists());
    assert!(ddir.path().join("var/lib/example/3.txt").exists());
    assert!(ddir.path().join("usr/share/doc/example/copyright").exists());
    assert!(ddir.path().join("usr/share/doc/example/changelog.Debian.gz").exists());
    assert!(ddir.path().join("usr/bin/example").exists());
    // changelog.Debian.gz starts with the gzip magic
    assert_eq!(
        &[0x1F, 0x8B],
        &fs::read(ddir.path().join("usr/share/doc/example/changelog.Debian.gz")).unwrap()[..2]
    );
}

#[test]
#[cfg(all(feature = "lzma"))]
fn run_cargo_deb_command_on_example_dir_with_variant() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let cmd_path = root.join(format!("target/debug/cargotodebian{}", std::env::consts::EXE_SUFFIX));
    assert!(cmd_path.exists());
    let cargo_dir = tempfile::tempdir().unwrap();
    let deb_path = cargo_dir.path().join("test.deb");
    let output = Command::new(cmd_path)
        .env("CARGO_TARGET_DIR", cargo_dir.path()) // otherwise tests overwrite each other
        .arg("--variant=debug")
        .arg("--no-strip")
        .arg(format!("--output={}", deb_path.display()))
        .arg(format!(
            "--manifest-path={}",
            root.join("example/Cargo.toml").display()
        ))
        .output()
        .unwrap();
    if !output.status.success() {
        panic!("Cmd failed: {}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    }

    // prints deb path on the last line
    let last_line = output.stdout[..output.stdout.len()-1].split(|&c| c==b'\n').last().unwrap();
    let printed_deb_path = Path::new(::std::str::from_utf8(last_line).unwrap());
    assert_eq!(printed_deb_path, deb_path);
    assert!(deb_path.exists());

    let ardir = tempfile::tempdir().unwrap();
    assert!(ardir.path().exists());
    assert!(Command::new("ar")
        .current_dir(ardir.path())
        .arg("-x")
        .arg(deb_path)
        .status().unwrap().success());

    assert_eq!("2.0\n", fs::read_to_string(ardir.path().join("debian-binary")).unwrap());
    assert!(ardir.path().join("data.tar.xz").exists());
    assert!(ardir.path().join("control.tar.xz").exists());

    let cdir = tempfile::tempdir().unwrap();
    assert!(Command::new("tar")
        .arg("xJf")
        .current_dir(cdir.path())
        .arg(ardir.path().join("control.tar.xz"))
        .status().unwrap().success());

    let control = fs::read_to_string(cdir.path().join("control")).unwrap();
    assert!(control.contains("Package: example-debug\n"), "Control is: {:?}", control);
    assert!(control.contains("Version: 0.1.0\n"));
    assert!(control.contains("Section: utils\n"));
    assert!(control.contains("Architecture: "));
    assert!(control.contains("Maintainer: cargotodebian developers <cargotodebian@example.invalid>\n"));

    let md5sums = fs::read_to_string(cdir.path().join("md5sums")).unwrap();
    assert!(md5sums.contains(" usr/bin/example\n"));
    assert!(md5sums.contains(" usr/share/doc/example-debug/changelog.Debian.gz\n"));
    assert!(md5sums.contains("83453jk345jk234jk234jk234jk2349s  var/lib/example/1.txt\n"));
    assert!(md5sums.contains("25j8s8sdf8g4891dsf0336m7w654m544  var/lib/example/2.txt\n"));
    assert!(md5sums.contains("835a3c46f2330925774ebf780aa74241  var/lib/example/4.txt\n"));
    assert!(md5sums.contains("2455967cef930e647146a8c762199ed3  usr/share/doc/example-debug/copyright\n"));

    let ddir = tempfile::tempdir().unwrap();
    assert!(Command::new("tar")
        .arg("xJf")
        .current_dir(ddir.path())
        .arg(ardir.path().join("data.tar.xz"))
        .status().unwrap().success());

    assert!(ddir.path().join("var/lib/example/1.txt").exists());
    assert!(ddir.path().join("var/lib/example/2.txt").exists());
    assert!(ddir.path().join("var/lib/example/4.txt").exists());
    assert!(ddir.path().join("usr/share/doc/example-debug/copyright").exists());
    assert!(ddir.path().join("usr/share/doc/example-debug/changelog.Debian.gz").exists());
    assert!(ddir.path().join("usr/bin/example").exists());
}

#[test]
#[cfg(all(feature = "lzma", target_os = "linux"))]
fn run_cargo_deb_command_on_example_dir_with_version() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let cmd_path = root.join("target/debug/cargotodebian");
    assert!(cmd_path.exists());
    let cargo_dir = tempfile::tempdir().unwrap();
    let deb_path = cargo_dir.path().join("test.deb");
    let output = Command::new(cmd_path)
        .env("CARGO_TARGET_DIR", cargo_dir.path()) // otherwise tests overwrite each other
        .arg(format!("--output={}", deb_path.display()))
        .arg(format!(
            "--manifest-path={}",
            root.join("example/Cargo.toml").display()
        ))
        .arg(format!("--deb-version=my-custom-version"))
        .output().unwrap();
    assert!(output.status.success());

    // prints deb path on the last line
    let last_line = output.stdout[..output.stdout.len() - 1].split(|&c| c == b'\n').last().unwrap();
    let deb_path = Path::new(::std::str::from_utf8(last_line).unwrap());
    assert!(deb_path.exists());

    let ardir = tempfile::tempdir().unwrap();
    assert!(ardir.path().exists());
    assert!(Command::new("ar")
        .current_dir(ardir.path())
        .arg("-x")
        .arg(deb_path)
        .status().unwrap().success());

    assert_eq!("2.0\n", fs::read_to_string(ardir.path().join("debian-binary")).unwrap());
    assert!(ardir.path().join("data.tar.xz").exists());
    assert!(ardir.path().join("control.tar.xz").exists());

    let cdir = tempfile::tempdir().unwrap();
    assert!(Command::new("tar")
        .arg("xf")
        .current_dir(cdir.path())
        .arg(ardir.path().join("control.tar.xz"))
        .status().unwrap().success());

    let control = fs::read_to_string(cdir.path().join("control")).unwrap();
    assert!(control.contains("Package: example\n"));
    assert!(control.contains("Version: my-custom-version\n"));
    assert!(control.contains("Section: utils\n"));
    assert!(control.contains("Architecture: "));
    assert!(control.contains("Maintainer: cargotodebian developers <cargotodebian@example.invalid>\n"));

    let md5sums = fs::read_to_string(cdir.path().join("md5sums")).unwrap();
    assert!(md5sums.contains(" usr/bin/example\n"));
    assert!(md5sums.contains(" usr/share/doc/example/changelog.Debian.gz\n"));
    assert!(md5sums.contains("83453jk345jk234jk234jk234jk2349s  var/lib/example/1.txt\n"));
    assert!(md5sums.contains("25j8s8sdf8g4891dsf0336m7w654m544  var/lib/example/2.txt\n"));
    assert!(md5sums.contains("345k35a3459ag8f8d9s74k3k33439sdd  var/lib/example/3.txt\n"));
    assert!(md5sums.contains("9l0l2l53hk34jk3trg721kggfd944mm4  usr/share/doc/example/copyright\n"), "has:\n{}", md5sums);

    let ddir = tempfile::tempdir().unwrap();
    assert!(Command::new("tar")
        .arg("xJf")
        .current_dir(ddir.path())
        .arg(ardir.path().join("data.tar.xz"))
        .status().unwrap().success());

    assert!(ddir.path().join("var/lib/example/1.txt").exists());
    assert!(ddir.path().join("var/lib/example/2.txt").exists());
    assert!(ddir.path().join("var/lib/example/3.txt").exists());
    assert!(ddir.path().join("usr/share/doc/example/copyright").exists());
    assert!(ddir.path().join("usr/share/doc/example/changelog.Debian.gz").exists());
    assert!(ddir.path().join("usr/bin/example").exists());
    // changelog.Debian.gz starts with the gzip magic
    assert_eq!(
        &[0x1F, 0x8B],
        &fs::read(ddir.path().join("usr/share/doc/example/changelog.Debian.gz")).unwrap()[..2]
    );
}

