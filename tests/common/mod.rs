use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::PathBuf;

use tempfile::TempDir;

// XXX Want a global lock on environment variable mutation to prevent interleaving
// tests from stepping on each other.
// https://github.com/rust-lang/rust/issues/43155#issuecomment-315543432 should
// work, but I can't get access to the `lazy_static!` macro in this file to work.
struct EnvVarState {
    changed: HashMap<OsString, Option<OsString>>,
}

impl Drop for EnvVarState {
    fn drop(&mut self) {
        self.changed.iter().for_each(|(k, v)| match &v {
            Some(original_v) => env::set_var(&k, original_v),
            None => env::remove_var(&k),
        });
    }
}

impl EnvVarState {
    fn new() -> Self {
        Self {
            changed: HashMap::new(),
        }
    }

    fn change(&mut self, k: &OsStr, v: Option<&OsStr>) {
        if !self.changed.contains_key(k) {
            let original_v = env::var_os(k);
            self.changed.insert(k.to_os_string(), original_v);
        }
        match v {
            Some(new_v) => env::set_var(k, new_v),
            None => env::remove_var(k),
        }
    }
}

fn touch_file(path: PathBuf) -> PathBuf {
    let file = File::create(&path).unwrap();
    file.sync_all().unwrap();
    path
}

pub struct EnvState {
    _dir1: TempDir,
    _dir2: TempDir,
    _env_changes: EnvVarState,
    pub python27: PathBuf,
    pub python36: PathBuf,
    pub python37: PathBuf,
}

impl EnvState {
    pub fn new() -> Self {
        let dir1 = TempDir::new().unwrap();
        let dir2 = TempDir::new().unwrap();

        let python27 = touch_file(dir1.path().join("python2.7"));
        let python36 = touch_file(dir1.path().join("python3.6"));
        touch_file(dir2.path().join("python3.6"));
        let python37 = touch_file(dir2.path().join("python3.7"));

        let new_path = env::join_paths([dir1.path(), dir2.path()].iter()).unwrap();
        let mut env_changes = EnvVarState::new();
        env_changes.change(OsStr::new("PATH"), Some(&new_path));
        env_changes.change(OsStr::new("VIRTUAL_ENV"), None);

        Self {
            _dir1: dir1,
            _dir2: dir2,
            _env_changes: env_changes,
            python27,
            python36,
            python37,
        }
    }
}