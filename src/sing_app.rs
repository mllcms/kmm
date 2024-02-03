use std::fs::File;
use std::io::Write;
use std::os::windows::fs::OpenOptionsExt;
use std::thread::sleep;
use std::time::Duration;
use std::{env, io, process};

pub struct SingApp {
    #[allow(unused)]
    lock: File,
}

#[cfg(target_os = "windows")]
impl SingApp {
    fn lock() -> io::Result<File> {
        let path = env::current_exe().unwrap();
        File::options()
            .read(true)
            .write(true)
            .create(true)
            .share_mode(1) // 保留读取权限
            .open(path.with_extension("lock"))
    }

    pub fn run() -> Self {
        match Self::lock() {
            Ok(lock) => Self { lock },
            Err(err) => {
                eprintln!("{err}");
                process::exit(0)
            }
        }
    }

    pub fn run_current() -> io::Result<Self> {
        match Self::lock() {
            Ok(mut lock) => {
                write!(&mut lock, "{}", process::id())?;
                Ok(Self { lock })
            }
            Err(_) => {
                let path = env::current_exe().unwrap().with_extension("lock");
                let file = File::options().read(true).open(path)?;
                let pid = io::read_to_string(file)?;

                process::Command::new("taskkill")
                    .arg("/F") // 使用 /F 标志强制杀死进程
                    .arg("/PID")
                    .arg(pid)
                    .output()?;
                sleep(Duration::from_millis(500));
                let mut lock = Self::lock()?;
                write!(&mut lock, "{}", process::id())?;
                Ok(Self { lock })
            }
        }
    }
}
