use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for attempt in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "zvvnmod-installer-test-{}-{nonce}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create test directory: {error}"),
            }
        }
        panic!("could not allocate test directory")
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_executable(path: &Path, body: &str) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o755)
        .open(path)
        .unwrap();
    file.write_all(body.as_bytes()).unwrap();
    file.sync_all().unwrap();
    drop(file);
}

#[test]
fn installer_prints_registry_stable_xdg_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .arg("--print-path")
        .env_remove("ZVVNMOD_MONGOL_NORM_PATH")
        .env("XDG_DATA_HOME", "/tmp/zvvnmod-installer-cli-xdg")
        .env_remove("HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "/tmp/zvvnmod-installer-cli-xdg/zvvnmod-utn57/mongol-norm/0.0.4/site\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn installer_uses_hash_locked_pip_target_and_replaces_from_staging() {
    let temp = TempDir::new();
    let install_path = temp.0.join("site");
    let log_path = temp.0.join("python.log");
    let fake_python = temp.0.join("python");
    write_executable(
        &fake_python,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {log}\ncase \" $* \" in\n  *' -m pip '*)\n    target=\n    previous=\n    for argument in \"$@\"; do\n      if [ \"$previous\" = target ]; then target=$argument; break; fi\n      if [ \"$argument\" = --target ]; then previous=target; fi\n    done\n    mkdir -p \"$target/mongol_norm\" \"$target/mongol_norm-0.0.4.dist-info\"\n    printf '%s\\n' installed > \"$target/marker\"\n    exit 0\n    ;;\nesac\ncat >/dev/null\nprintf '\\341\\240\\244\\341\\240\\213\\342\\200\\215'\n",
            log = log_path.display()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .env("ZVVNMOD_MONGOL_NORM_PATH", &install_path)
        .env("ZVVNMOD_MONGOL_NORM_PYTHON", &fake_python)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(install_path.join("marker")).unwrap(),
        "installed\n"
    );
    let log = fs::read_to_string(log_path).unwrap();
    assert!(log.contains("-I -m pip install"), "log: {log}");
    assert!(log.contains("--require-hashes"), "log: {log}");
    assert!(log.contains("--no-deps"), "log: {log}");
    assert!(log.contains("--target"), "log: {log}");
    assert!(!install_path.join("requirements-mongol-norm.txt").exists());
}

#[test]
fn installer_prints_registry_stable_home_fallback() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .arg("--print-path")
        .env_remove("ZVVNMOD_MONGOL_NORM_PATH")
        .env_remove("XDG_DATA_HOME")
        .env("HOME", "/tmp/zvvnmod-installer-cli-home")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "/tmp/zvvnmod-installer-cli-home/.local/share/zvvnmod-utn57/mongol-norm/0.0.4/site\n"
    );
}

#[test]
fn installer_ignores_relative_xdg_data_home_and_uses_absolute_home() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .arg("--print-path")
        .env_remove("ZVVNMOD_MONGOL_NORM_PATH")
        .env("XDG_DATA_HOME", "relative-data")
        .env("HOME", "/tmp/zvvnmod-installer-cli-relative-xdg-home")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "/tmp/zvvnmod-installer-cli-relative-xdg-home/.local/share/zvvnmod-utn57/mongol-norm/0.0.4/site\n"
    );
}

#[test]
fn installer_rejects_relative_explicit_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .arg("--print-path")
        .env("ZVVNMOD_MONGOL_NORM_PATH", "relative-site")
        .env("XDG_DATA_HOME", "/tmp/zvvnmod-installer-cli-unused-xdg")
        .env("HOME", "/tmp/zvvnmod-installer-cli-unused-home")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("absolute"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn installer_rejects_relative_xdg_and_home_bases() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .arg("--print-path")
        .env_remove("ZVVNMOD_MONGOL_NORM_PATH")
        .env("XDG_DATA_HOME", "relative-data")
        .env("HOME", "relative-home")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("absolute"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn installer_rejects_a_symlink_destination_before_starting_python() {
    let temp = TempDir::new();
    let real = temp.0.join("real");
    let install_path = temp.0.join("site");
    fs::create_dir(&real).unwrap();
    std::os::unix::fs::symlink(&real, &install_path).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .env("ZVVNMOD_MONGOL_NORM_PATH", &install_path)
        .env("ZVVNMOD_MONGOL_NORM_PYTHON", "/definitely/missing/python")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("symlink install destination"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(install_path.is_symlink());
}

#[test]
fn failed_pip_preserves_the_previous_installation() {
    let temp = TempDir::new();
    let install_path = temp.0.join("site");
    fs::create_dir(&install_path).unwrap();
    fs::write(install_path.join("marker"), "previous\n").unwrap();
    let fake_python = temp.0.join("failing-python");
    write_executable(&fake_python, "#!/bin/sh\nexit 17\n");

    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .env("ZVVNMOD_MONGOL_NORM_PATH", &install_path)
        .env("ZVVNMOD_MONGOL_NORM_PYTHON", &fake_python)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(install_path.join("marker")).unwrap(),
        "previous\n"
    );
    assert!(
        fs::read_dir(&temp.0).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("stage")),
        "staging directory was not cleaned up"
    );
}

#[test]
fn installer_rejects_extra_arguments_without_starting_python() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-install-mongol-norm"))
        .arg("unexpected")
        .env("ZVVNMOD_MONGOL_NORM_PYTHON", "/definitely/missing/python")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "usage: zvvnmod-install-mongol-norm [--print-path]\n"
    );
}
