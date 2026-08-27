use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use zvvnmod_utn57::mongol_norm_install_path;

const REQUIREMENTS: &str = include_str!("../../requirements-mongol-norm.txt");
const BRIDGE_SCRIPT: &str = include_str!("../../scripts/mongol_norm_positioned.py");
const VALIDATION_INPUT: &[u8] =
    b"{\"protocol\":1,\"records\":[{\"unit\":\"O\",\"position\":\"init\"}]}\n";
const VALIDATION_OUTPUT: &[u8] = "\u{1824}\u{180b}\u{200d}".as_bytes();

struct TemporaryDirectory(Option<PathBuf>);

impl TemporaryDirectory {
    fn create(parent: &Path, label: &str) -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        for attempt in 0..100 {
            let path = parent.join(format!(
                ".mongol-norm-{label}-{}-{nonce}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(Some(path))),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("could not create {}: {error}", path.display())),
            }
        }
        Err(format!(
            "could not allocate a temporary directory under {}",
            parent.display()
        ))
    }

    fn path(&self) -> &Path {
        self.0.as_deref().expect("temporary directory is present")
    }

    fn take(&mut self) -> PathBuf {
        self.0.take().expect("temporary directory is present")
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn python_command() -> OsString {
    std::env::var_os("ZVVNMOD_MONGOL_NORM_PYTHON")
        .or_else(|| std::env::var_os("PYTHON"))
        .unwrap_or_else(|| OsString::from("python3"))
}

fn validate_destination(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "refusing symlink install destination: {}",
            path.display()
        )),
        Ok(metadata) if !metadata.is_dir() => Err(format!(
            "refusing non-directory install destination: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("could not inspect {}: {error}", path.display())),
    }
}

fn install_with_pip(python: &OsStr, stage: &Path) -> Result<(), String> {
    let requirements = stage.join("requirements-mongol-norm.txt");
    fs::write(&requirements, REQUIREMENTS)
        .map_err(|error| format!("could not write {}: {error}", requirements.display()))?;

    let status = Command::new(python)
        .args(["-I", "-m", "pip", "install", "--disable-pip-version-check"])
        .arg("--require-hashes")
        .arg("--no-deps")
        .arg("--upgrade")
        .arg("--target")
        .arg(stage)
        .arg("-r")
        .arg(&requirements)
        .status()
        .map_err(|error| {
            format!(
                "could not start Python command {:?}: {error}",
                python.to_string_lossy()
            )
        })?;
    if !status.success() {
        return Err(format!(
            "pip failed with status {}",
            status
                .code()
                .map_or_else(|| "signal".to_owned(), |code| code.to_string())
        ));
    }
    fs::remove_file(&requirements)
        .map_err(|error| format!("could not remove {}: {error}", requirements.display()))?;
    Ok(())
}

fn abort_validation(child: &mut Child, error: String) -> String {
    let kill_error = child.kill().err();
    let wait_error = child.wait().err();
    match (kill_error, wait_error) {
        (None, None) => error,
        (Some(kill_error), None) => {
            format!("{error}; could not kill staged validation: {kill_error}")
        }
        (None, Some(wait_error)) => {
            format!("{error}; could not reap staged validation: {wait_error}")
        }
        (Some(kill_error), Some(wait_error)) => format!(
            "{error}; could not kill staged validation: {kill_error}; could not reap staged validation: {wait_error}"
        ),
    }
}

fn validate_install(python: &OsStr, stage: &Path) -> Result<(), String> {
    validate_install_with_input(python, stage, VALIDATION_INPUT)
}

fn validate_install_with_input(
    python: &OsStr,
    stage: &Path,
    validation_input: &[u8],
) -> Result<(), String> {
    let mut child = Command::new(python)
        .args(["-I", "-S", "-c"])
        .arg(BRIDGE_SCRIPT)
        .arg(stage)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start staged validation: {error}"))?;
    let mut stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            return Err(abort_validation(
                &mut child,
                "staged validation stdin was unavailable".to_owned(),
            ));
        }
    };
    if let Err(error) = stdin.write_all(validation_input) {
        drop(stdin);
        return Err(abort_validation(
            &mut child,
            format!("could not write staged validation input: {error}"),
        ));
    }
    drop(stdin);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("could not wait for staged validation: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "staged validation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim_end()
        ));
    }
    if output.stdout != VALIDATION_OUTPUT {
        return Err("staged validation returned unexpected O:init output".to_owned());
    }
    Ok(())
}

fn replace_destination(stage: TemporaryDirectory, install_path: &Path) -> Result<(), String> {
    replace_destination_with(stage, install_path, |from, to| fs::rename(from, to))
}

fn replace_destination_with(
    mut stage: TemporaryDirectory,
    install_path: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> Result<(), String> {
    validate_destination(install_path)?;
    let parent = install_path
        .parent()
        .ok_or_else(|| format!("install path has no parent: {}", install_path.display()))?;
    let backup_path = if install_path.exists() {
        let mut backup = TemporaryDirectory::create(parent, "backup")?;
        let empty_backup = backup.path().to_path_buf();
        fs::remove_dir(&empty_backup)
            .map_err(|error| format!("could not prepare {}: {error}", empty_backup.display()))?;
        rename(install_path, &empty_backup).map_err(|error| {
            format!(
                "could not move {} to {}: {error}",
                install_path.display(),
                empty_backup.display()
            )
        })?;
        Some(backup.take())
    } else {
        None
    };

    let staged_path = stage.path().to_path_buf();
    if let Err(error) = rename(&staged_path, install_path) {
        let move_error = format!(
            "could not move {} to {}: {error}",
            staged_path.display(),
            install_path.display()
        );
        if let Some(backup_path) = backup_path.as_ref() {
            if let Err(restore_error) = rename(backup_path, install_path) {
                return Err(format!(
                    "{move_error}; could not restore previous installation from {}: {restore_error}; previous installation preserved at {}",
                    backup_path.display(),
                    backup_path.display()
                ));
            }
        }
        return Err(move_error);
    }
    let _ = stage.take();

    if let Some(backup_path) = backup_path {
        fs::remove_dir_all(&backup_path)
            .map_err(|error| format!("could not remove {}: {error}", backup_path.display()))?;
    }
    Ok(())
}

fn install() -> Result<PathBuf, String> {
    let install_path = mongol_norm_install_path().map_err(|error| error.to_string())?;
    validate_destination(&install_path)?;
    let parent = install_path
        .parent()
        .ok_or_else(|| format!("install path has no parent: {}", install_path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("could not create {}: {error}", parent.display()))?;

    let stage = TemporaryDirectory::create(parent, "stage")?;
    let python = python_command();
    install_with_pip(&python, stage.path())?;
    validate_install(&python, stage.path())?;
    replace_destination(stage, &install_path)?;
    Ok(install_path)
}

fn run() -> Result<(), String> {
    match std::env::args_os().skip(1).collect::<Vec<_>>().as_slice() {
        [argument] if argument == "--print-path" => {
            println!(
                "{}",
                mongol_norm_install_path()
                    .map_err(|error| error.to_string())?
                    .display()
            );
            Ok(())
        }
        [] => {
            let install_path = install()?;
            println!("mongol-norm 0.0.4 installed and verified.");
            println!("Install path: {}", install_path.display());
            Ok(())
        }
        _ => Err("usage: zvvnmod-install-mongol-norm [--print-path]".to_owned()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_write_failure_kills_and_reaps_spawned_child() {
        use std::os::unix::fs::PermissionsExt;

        let parent = TemporaryDirectory::create(&std::env::temp_dir(), "test-validation").unwrap();
        let fake_python = parent.path().join("python");
        let pid_path = parent.path().join("validation.pid");
        fs::write(
            &fake_python,
            format!(
                "#!/bin/sh\nexec 0<&-\nprintf '%s' $$ > {}\nsleep 60\n",
                pid_path.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_python).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_python, permissions).unwrap();
        let input = vec![b'x'; 1024 * 1024];

        let error = validate_install_with_input(fake_python.as_os_str(), parent.path(), &input)
            .unwrap_err();

        assert!(
            error.contains("could not write staged validation input"),
            "{error}"
        );
        let pid = fs::read_to_string(pid_path).unwrap();
        assert!(!pid.is_empty(), "validation child did not start");
        assert!(
            !error.contains("could not kill staged validation")
                && !error.contains("could not reap staged validation"),
            "validation child cleanup failed: {error}"
        );
    }

    #[test]
    fn validation_setup_failure_kills_and_reaps_child() {
        let mut child = Command::new("sh")
            .args(["-c", "sleep 60"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let error = abort_validation(&mut child, "injected setup failure".to_owned());

        assert_eq!(error, "injected setup failure");
        assert!(child.try_wait().unwrap().is_some(), "child was not reaped");
    }

    #[test]
    fn failed_stage_move_and_restore_preserve_and_report_backup() {
        let parent = TemporaryDirectory::create(&std::env::temp_dir(), "test-parent").unwrap();
        let install_path = parent.path().join("site");
        fs::create_dir(&install_path).unwrap();
        fs::write(install_path.join("marker"), "previous\n").unwrap();
        let stage = TemporaryDirectory::create(parent.path(), "stage").unwrap();
        fs::write(stage.path().join("marker"), "staged\n").unwrap();
        let mut rename_count = 0;

        let error = replace_destination_with(stage, &install_path, |from, to| {
            rename_count += 1;
            if rename_count == 1 {
                fs::rename(from, to)
            } else {
                Err(io::Error::other(format!("injected rename {rename_count}")))
            }
        })
        .unwrap_err();

        assert!(!install_path.exists());
        let backup_path = fs::read_dir(parent.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains("backup")
            })
            .expect("backup path was preserved");
        assert_eq!(
            fs::read_to_string(backup_path.join("marker")).unwrap(),
            "previous\n"
        );
        assert!(
            error.contains(&backup_path.display().to_string()),
            "{error}"
        );
        assert!(error.contains("injected rename 3"), "{error}");

        fs::remove_dir_all(&backup_path).unwrap();
    }
}
