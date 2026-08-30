use crate::{
    convert_zvvnmod_run, mongol_norm_install_path, Utn57ConversionError,
    Utn57PositionedWrittenUnit, ZvvnmodCode,
};
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

const BRIDGE_SCRIPT: &str = include_str!("../scripts/mongol_norm_positioned.py");
const PYTHON_ENV: &str = "ZVVNMOD_MONGOL_NORM_PYTHON";

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const OUTPUT_LIMIT: usize = 1024 * 1024;
const CLEANUP_TIMEOUT: Duration = Duration::from_millis(500);
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Failure in the external `mongol-norm` command pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MongolNormCommandError {
    /// Rust ZVVNMOD → positioned-unit conversion failed.
    Conversion(Utn57ConversionError),
    /// The configured Python executable could not be started.
    Spawn { program: String, message: String },
    /// The configured mongol-norm install directory is unavailable.
    PythonPath { message: String },
    /// The positioned-unit payload could not be sent to Python.
    WriteInput { message: String },
    /// The Python bridge exceeded its bounded execution time.
    Timeout { duration: Duration },
    /// The Python bridge returned a non-zero exit status or could not be waited on.
    CommandFailed { code: Option<i32>, stderr: String },
    /// Successful command output was not UTF-8.
    InvalidUtf8 { message: String },
}

impl fmt::Display for MongolNormCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conversion(error) => error.fmt(formatter),
            Self::Spawn { program, message } => {
                write!(
                    formatter,
                    "could not start Python command {program:?}: {message}"
                )
            }
            Self::PythonPath { message } => {
                write!(formatter, "invalid mongol-norm module path: {message}")
            }
            Self::WriteInput { message } => {
                write!(
                    formatter,
                    "could not send positioned units to Python: {message}"
                )
            }
            Self::Timeout { duration } => write!(
                formatter,
                "mongol-norm command timed out after {:.3} seconds",
                duration.as_secs_f64()
            ),
            Self::CommandFailed { code, stderr } => write!(
                formatter,
                "mongol-norm command failed with status {}: {}",
                code.map_or_else(|| "signal".to_owned(), |value| value.to_string()),
                stderr.trim_end()
            ),
            Self::InvalidUtf8 { message } => {
                write!(formatter, "mongol-norm returned invalid UTF-8: {message}")
            }
        }
    }
}

impl Error for MongolNormCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Conversion(error) => Some(error),
            _ => None,
        }
    }
}

impl From<Utn57ConversionError> for MongolNormCommandError {
    fn from(error: Utn57ConversionError) -> Self {
        Self::Conversion(error)
    }
}

fn default_python() -> OsString {
    std::env::var_os(PYTHON_ENV).unwrap_or_else(|| OsString::from("python3"))
}

fn positioned_payload(units: &[Utn57PositionedWrittenUnit]) -> String {
    let mut payload = String::from("{\"protocol\":1,\"records\":[");
    for (index, unit) in units.iter().enumerate() {
        if index != 0 {
            payload.push(',');
        }
        payload.push_str("{\"unit\":\"");
        payload.push_str(unit.written_unit.contract_name());
        payload.push_str("\",\"position\":\"");
        payload.push_str(unit.position.contract_name());
        payload.push_str("\"}");
    }
    payload.push_str("]}");
    payload
}

struct Worker<T> {
    receiver: mpsc::Receiver<T>,
    handle: Option<thread::JoinHandle<()>>,
    result: Option<T>,
}

impl<T: Send + 'static> Worker<T> {
    fn spawn(task: impl FnOnce() -> T + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let result = task();
            let _ = sender.send(result);
        });
        Self {
            receiver,
            handle: Some(handle),
            result: None,
        }
    }

    fn poll(&mut self) {
        if self
            .handle
            .as_ref()
            .is_some_and(|handle| handle.is_finished())
        {
            // `is_finished` makes this join non-blocking. Read the channel after
            // joining so a just-finished sender cannot be mistaken for a panic.
            let _ = self.handle.take().expect("worker handle present").join();
        }
        if self.result.is_none() {
            if let Ok(result) = self.receiver.try_recv() {
                self.result = Some(result);
            }
        }
    }

    fn is_collected(&self) -> bool {
        self.handle.is_none()
    }
}

fn read_bounded(mut reader: impl Read, stream: &str) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("could not read Python {stream}: {error}"))?;
        if count == 0 {
            return Ok(output);
        }
        if output.len().saturating_add(count) > OUTPUT_LIMIT {
            return Err(format!(
                "Python {stream} exceeded the {OUTPUT_LIMIT}-byte capture limit"
            ));
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) {
    use std::os::raw::c_int;

    unsafe extern "C" {
        fn kill(pid: c_int, signal: c_int) -> c_int;
    }

    let process_group = match c_int::try_from(child.id()) {
        Ok(pid) => -pid,
        Err(_) => {
            let _ = child.kill();
            return;
        }
    };
    // SAFETY: `process_group` is the negative id of the dedicated process group
    // assigned before spawn. POSIX `kill(-pgid, SIGKILL)` does not dereference
    // memory and targets only that group. Failure is harmless and followed by a
    // best-effort direct-child kill; all subsequent waits remain bounded.
    let _ = unsafe { kill(process_group, 9) };
    let _ = child.kill();
}

#[cfg(not(unix))]
fn terminate_process_tree(child: &mut Child) {
    let _ = child.kill();
}

/// Normalize positioned units with an explicit Python executable, install path, and timeout.
fn normalize_positioned_with_mongol_norm_python_at(
    units: &[Utn57PositionedWrittenUnit],
    python: impl AsRef<OsStr>,
    module_path: impl AsRef<Path>,
    timeout: Duration,
) -> Result<String, MongolNormCommandError> {
    let python = python.as_ref();
    let mut command = Command::new(python);
    command
        .args(["-I", "-S", "-c"])
        .arg(BRIDGE_SCRIPT)
        .arg(module_path.as_ref())
        .env_remove("PYTHONPATH");
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| MongolNormCommandError::Spawn {
            program: python.to_string_lossy().into_owned(),
            message: error.to_string(),
        })?;

    let payload = positioned_payload(units);
    let Some(mut stdin) = child.stdin.take() else {
        terminate_process_tree(&mut child);
        return Err(MongolNormCommandError::WriteInput {
            message: "Python stdin pipe was unavailable".to_owned(),
        });
    };
    let Some(stdout) = child.stdout.take() else {
        terminate_process_tree(&mut child);
        return Err(MongolNormCommandError::CommandFailed {
            code: None,
            stderr: "Python stdout pipe was unavailable".to_owned(),
        });
    };
    let Some(stderr) = child.stderr.take() else {
        terminate_process_tree(&mut child);
        return Err(MongolNormCommandError::CommandFailed {
            code: None,
            stderr: "Python stderr pipe was unavailable".to_owned(),
        });
    };

    let mut writer = Worker::spawn(move || {
        let result = stdin
            .write_all(payload.as_bytes())
            .map_err(|error| error.to_string());
        drop(stdin);
        result
    });
    let mut stdout_reader = Worker::spawn(move || read_bounded(stdout, "stdout"));
    let mut stderr_reader = Worker::spawn(move || read_bounded(stderr, "stderr"));

    let deadline = Instant::now()
        .checked_add(timeout)
        .unwrap_or_else(Instant::now);
    let mut cleanup_deadline = None;
    let mut status: Option<ExitStatus> = None;
    let mut pipeline_error: Option<MongolNormCommandError> = None;

    loop {
        writer.poll();
        stdout_reader.poll();
        stderr_reader.poll();

        if status.is_none() {
            match child.try_wait() {
                Ok(Some(exit_status)) => {
                    if !exit_status.success() {
                        terminate_process_tree(&mut child);
                        cleanup_deadline.get_or_insert_with(|| Instant::now() + CLEANUP_TIMEOUT);
                    }
                    status = Some(exit_status);
                }
                Ok(None) => {}
                Err(error) => {
                    pipeline_error.get_or_insert_with(|| MongolNormCommandError::CommandFailed {
                        code: None,
                        stderr: format!("could not wait for Python command: {error}"),
                    });
                }
            }
        }

        if pipeline_error.is_none() {
            if let Some(Err(message)) = writer.result.as_ref() {
                pipeline_error = Some(MongolNormCommandError::WriteInput {
                    message: message.clone(),
                });
            } else if writer.is_collected() && writer.result.is_none() {
                pipeline_error = Some(MongolNormCommandError::WriteInput {
                    message: "stdin writer thread terminated unexpectedly".to_owned(),
                });
            }
        }

        // Inspect both readers before acting so one reader failure never causes the
        // other worker to be returned from or detached asymmetrically.
        if pipeline_error.is_none() {
            let stdout_error = match stdout_reader.result.as_ref() {
                Some(Err(message)) => Some(message.clone()),
                None if stdout_reader.is_collected() => {
                    Some("stdout reader thread terminated unexpectedly".to_owned())
                }
                _ => None,
            };
            let stderr_error = match stderr_reader.result.as_ref() {
                Some(Err(message)) => Some(message.clone()),
                None if stderr_reader.is_collected() => {
                    Some("stderr reader thread terminated unexpectedly".to_owned())
                }
                _ => None,
            };
            if stdout_error.is_some() || stderr_error.is_some() {
                let message = [stdout_error, stderr_error]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("; ");
                pipeline_error = Some(MongolNormCommandError::CommandFailed {
                    code: status.as_ref().and_then(ExitStatus::code),
                    stderr: message,
                });
            }
        }

        if pipeline_error.is_some() {
            terminate_process_tree(&mut child);
            cleanup_deadline.get_or_insert_with(|| Instant::now() + CLEANUP_TIMEOUT);
        }

        let workers_collected =
            writer.is_collected() && stdout_reader.is_collected() && stderr_reader.is_collected();
        if status.is_some() && workers_collected {
            break;
        }

        let now = Instant::now();
        if now >= deadline && pipeline_error.is_none() {
            pipeline_error = Some(MongolNormCommandError::Timeout { duration: timeout });
            terminate_process_tree(&mut child);
            cleanup_deadline = Some(now + CLEANUP_TIMEOUT);
        }
        if cleanup_deadline.is_some_and(|cleanup| now >= cleanup) {
            // Never block in `wait` or `join` after a failed kill. If cleanup could
            // not finish, report the bounded deadline rather than a reader error
            // while silently detaching its peer.
            return Err(match pipeline_error {
                Some(MongolNormCommandError::Timeout { duration }) => {
                    MongolNormCommandError::Timeout { duration }
                }
                _ => MongolNormCommandError::Timeout { duration: timeout },
            });
        }
        thread::sleep(POLL_INTERVAL);
    }

    if let Some(error) = pipeline_error {
        return Err(error);
    }
    let status = status.expect("collected process has an exit status");
    let stdout = stdout_reader
        .result
        .expect("collected stdout worker has a result")
        .map_err(|message| MongolNormCommandError::CommandFailed {
            code: status.code(),
            stderr: message,
        })?;
    let stderr = stderr_reader
        .result
        .expect("collected stderr worker has a result")
        .map_err(|message| MongolNormCommandError::CommandFailed {
            code: status.code(),
            stderr: message,
        })?;
    writer
        .result
        .expect("collected stdin worker has a result")
        .map_err(|message| MongolNormCommandError::WriteInput { message })?;
    if !status.success() {
        return Err(MongolNormCommandError::CommandFailed {
            code: status.code(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        });
    }
    String::from_utf8(stdout).map_err(|error| MongolNormCommandError::InvalidUtf8 {
        message: error.to_string(),
    })
}

/// Normalize positioned written units by invoking an explicit Python executable.
pub fn normalize_positioned_with_mongol_norm_python(
    units: &[Utn57PositionedWrittenUnit],
    python: impl AsRef<OsStr>,
) -> Result<String, MongolNormCommandError> {
    let module_path =
        mongol_norm_install_path().map_err(|error| MongolNormCommandError::PythonPath {
            message: error.to_string(),
        })?;
    normalize_positioned_with_mongol_norm_python_at(units, python, module_path, COMMAND_TIMEOUT)
}

/// Normalize positioned written units using the configured Python command.
pub fn normalize_positioned_with_mongol_norm(
    units: &[Utn57PositionedWrittenUnit],
) -> Result<String, MongolNormCommandError> {
    normalize_positioned_with_mongol_norm_python(units, default_python())
}

/// Convert a ZVVNMOD PUA string to canonical Mongolian Unicode through the external
/// `mongol-norm` command.
pub fn convert_zvvnmod_text_with_mongol_norm(
    input: &str,
) -> Result<String, MongolNormCommandError> {
    let codes: Vec<_> = input
        .chars()
        .map(|character| ZvvnmodCode(character as u32))
        .collect();
    let units = convert_zvvnmod_run(&codes)?;
    normalize_positioned_with_mongol_norm(&units)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    static EXECUTABLE_FIXTURE_LOCK: Mutex<()> = Mutex::new(());

    fn lock_executable_fixture() -> MutexGuard<'static, ()> {
        EXECUTABLE_FIXTURE_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            for attempt in 0..100 {
                let path = std::env::temp_dir().join(format!(
                    "zvvnmod-command-bridge-{label}-{}-{nonce}-{attempt}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("could not create temporary test directory: {error}"),
                }
            }
            panic!("could not allocate a unique temporary test directory")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_fake_distribution(
        root: &Path,
        module_version: &str,
        metadata_version: &str,
        marker: &str,
    ) {
        let package = root.join("mongol_norm");
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("__init__.py"),
            format!(
                "__version__ = {module_version:?}\nclass MongolianShaper:\n    def __init__(self, _language): pass\n    def normalize_positioned_written_units(self, _records): return {marker:?}\n"
            ),
        )
        .unwrap();
        let metadata = root.join(format!("mongol_norm-{metadata_version}.dist-info"));
        fs::create_dir_all(&metadata).unwrap();
        fs::write(
            metadata.join("METADATA"),
            format!("Metadata-Version: 2.1\nName: mongol-norm\nVersion: {metadata_version}\n"),
        )
        .unwrap();
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
    fn selected_install_cannot_be_shadowed_by_cwd_or_pythonpath() {
        let _fixture_guard = lock_executable_fixture();
        let temp = TempDir::new("isolated");
        let selected = temp.0.join("selected");
        let shadow = temp.0.join("shadow");
        write_fake_distribution(&selected, "0.0.4", "0.0.4", "selected");
        write_fake_distribution(&shadow, "0.0.4", "0.0.4", "shadow");
        let python = temp.0.join("python-with-hostile-environment");
        write_executable(
            &python,
            &format!(
                "#!/bin/sh\ncd {}\nPYTHONPATH={}\nexport PYTHONPATH\nexec python3 \"$@\"\n",
                shadow.display(),
                shadow.display()
            ),
        );

        let output = normalize_positioned_with_mongol_norm_python_at(
            &[],
            &python,
            &selected,
            Duration::from_secs(5),
        )
        .unwrap();

        assert_eq!(output, "selected");
    }

    #[test]
    fn wrong_distribution_metadata_version_is_rejected() {
        let temp = TempDir::new("wrong-metadata");
        let selected = temp.0.join("selected");
        write_fake_distribution(&selected, "0.0.4", "9.9.9", "unused");

        let error = normalize_positioned_with_mongol_norm_python_at(
            &[],
            "python3",
            &selected,
            Duration::from_secs(5),
        )
        .unwrap_err();

        match error {
            MongolNormCommandError::CommandFailed { stderr, .. } => {
                assert!(stderr.contains("9.9.9"), "stderr: {stderr}");
                assert!(stderr.contains("0.0.4"), "stderr: {stderr}");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn timeout_kills_and_reaps_python_process() {
        let _fixture_guard = lock_executable_fixture();
        let temp = TempDir::new("timeout");
        let selected = temp.0.join("selected");
        fs::create_dir_all(&selected).unwrap();
        let pid_file = temp.0.join("pid");
        let python = temp.0.join("hanging-python");
        write_executable(
            &python,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$$\" > {}\nexec sleep 60\n",
                pid_file.display()
            ),
        );
        let started = Instant::now();

        let error = normalize_positioned_with_mongol_norm_python_at(
            &[],
            &python,
            &selected,
            Duration::from_millis(150),
        )
        .unwrap_err();

        assert!(matches!(error, MongolNormCommandError::Timeout { .. }));
        assert!(started.elapsed() < Duration::from_secs(3));
        let pid = fs::read_to_string(pid_file).unwrap();
        assert!(
            !Path::new("/proc").join(pid.trim()).exists(),
            "timed-out child {pid:?} was not killed and reaped"
        );
    }

    #[test]
    fn timeout_kills_descendant_after_direct_child_exits() {
        let _fixture_guard = lock_executable_fixture();
        let temp = TempDir::new("descendant-timeout");
        let selected = temp.0.join("selected");
        fs::create_dir_all(&selected).unwrap();
        let pid_file = temp.0.join("descendant-pid");
        let python = temp.0.join("forking-python");
        write_executable(
            &python,
            &format!(
                "#!/bin/sh\ncat >/dev/null\nsleep 60 &\nprintf '%s\\n' \"$!\" > {}\nexit 0\n",
                pid_file.display()
            ),
        );
        let started = Instant::now();

        let error = normalize_positioned_with_mongol_norm_python_at(
            &[],
            &python,
            &selected,
            Duration::from_millis(150),
        )
        .unwrap_err();

        assert!(matches!(error, MongolNormCommandError::Timeout { .. }));
        assert!(started.elapsed() < Duration::from_secs(3));
        let pid = fs::read_to_string(pid_file).unwrap();
        let proc_path = Path::new("/proc").join(pid.trim());
        let reap_deadline = Instant::now() + Duration::from_secs(2);
        while proc_path.exists() && Instant::now() < reap_deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            !proc_path.exists(),
            "timed-out descendant {pid:?} was not terminated"
        );
    }

    #[test]
    fn stdout_capture_is_bounded() {
        let _fixture_guard = lock_executable_fixture();
        let temp = TempDir::new("stdout-limit");
        let selected = temp.0.join("selected");
        fs::create_dir_all(&selected).unwrap();
        let pid_file = temp.0.join("descendant-pid");
        let python = temp.0.join("verbose-python");
        write_executable(
            &python,
            &format!(
                "#!/bin/sh\nsleep 60 &\nprintf '%s\\n' \"$!\" > {}\ndd if=/dev/zero bs={} count=1 2>/dev/null\nexit 0\n",
                pid_file.display(),
                OUTPUT_LIMIT + 1,
            ),
        );

        let error = normalize_positioned_with_mongol_norm_python_at(
            &[],
            &python,
            &selected,
            Duration::from_secs(3),
        )
        .unwrap_err();

        match error {
            MongolNormCommandError::CommandFailed { stderr, .. } => {
                assert!(stderr.contains("stdout"), "stderr: {stderr}");
                assert!(stderr.contains("capture limit"), "stderr: {stderr}");
            }
            other => panic!("unexpected error: {other}"),
        }
        let pid = fs::read_to_string(pid_file).unwrap();
        let proc_path = Path::new("/proc").join(pid.trim());
        let reap_deadline = Instant::now() + Duration::from_secs(2);
        while proc_path.exists() && Instant::now() < reap_deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            !proc_path.exists(),
            "output-error descendant {pid:?} was not terminated"
        );
    }

    #[test]
    fn stderr_capture_is_bounded() {
        let _fixture_guard = lock_executable_fixture();
        let temp = TempDir::new("stderr-limit");
        let selected = temp.0.join("selected");
        fs::create_dir_all(&selected).unwrap();
        let python = temp.0.join("verbose-python");
        write_executable(
            &python,
            &format!(
                "#!/bin/sh\ndd if=/dev/zero bs={} count=1 1>&2 2>/dev/null\n",
                OUTPUT_LIMIT + 1
            ),
        );

        let error = normalize_positioned_with_mongol_norm_python_at(
            &[],
            &python,
            &selected,
            Duration::from_secs(3),
        )
        .unwrap_err();

        match error {
            MongolNormCommandError::CommandFailed { stderr, .. } => {
                assert!(stderr.contains("stderr"), "stderr: {stderr}");
                assert!(stderr.contains("capture limit"), "stderr: {stderr}");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
