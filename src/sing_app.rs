use std::{
    env,
    fs::File,
    io::{self, Write},
    os::windows::fs::OpenOptionsExt,
    process,
    thread::sleep,
    time::Duration,
};

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
            .truncate(true)
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

                let status = process::Command::new("taskkill")
                    .arg("/F") // 使用 /F 标志强制杀死进程
                    .arg("/PID")
                    .arg(pid)
                    .status()?;
                if !status.success() {
                    process::exit(0)
                }

                sleep(Duration::from_millis(500));
                let mut lock = Self::lock()?;
                write!(&mut lock, "{}", process::id())?;
                Ok(Self { lock })
            }
        }
    }
}
