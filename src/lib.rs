use windows::{core::PWSTR, Win32::{
    Foundation::{
        BOOL,
        TRUE,
        GetLastError,
        WAIT_OBJECT_0,
    },
    System::{
        Console::{
            SetConsoleTitleW,
            SetConsoleCtrlHandler,
            CTRL_C_EVENT,
        },
        Diagnostics::Debug::{
            FormatMessageW,
            FORMAT_MESSAGE_FROM_SYSTEM,
            FORMAT_MESSAGE_IGNORE_INSERTS,
        },
        Performance::{
            QueryPerformanceCounter,
            QueryPerformanceFrequency,
        },
        SystemServices::{
            LANG_NEUTRAL,
            SUBLANG_DEFAULT,
        },
        Threading::{
            CreateProcessW,
            GetExitCodeProcess,
            WaitForSingleObject,
            PROCESS_CREATION_FLAGS,
            PROCESS_INFORMATION,
            STARTUPINFOW,
            TerminateProcess,
        },
    },
}};

pub enum Status {
    SUCCESS,
    FAILURE,
}
impl From<Status> for i32 {
    fn from(status: Status) -> Self {
        match status {
            Status::SUCCESS => 0,
            Status::FAILURE => -1,
        }
    }
}
impl Copy for Status {}
impl Clone for Status {
    fn clone(&self) -> Self {
        *self
    }
}

pub fn set_console_title(title: &str) -> windows::core::Result<()> {
    unsafe {
        SetConsoleTitleW(PWSTR::from_raw(widestring::U16CString::from_str(title).unwrap().into_raw()))
    }
}

pub fn info(lable: &str, content: &str, state: Status) {
    match state {
        Status::SUCCESS => print!("\x1b[32;1m"),
        Status::FAILURE => print!("\x1b[31;1m"),
    }
    println!("{}\x1b[0m {}", lable, content)
}

pub fn pause_exit(state: Status) {
    info("Waiting", "enter any key to exit...", state);
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    std::process::exit(state.into());
}

fn makelangid(p: u32, s: u32) -> u32{
    (s << 10) | p
}

fn get_error_message() -> String {
    let mut result: Vec<u16> = vec![0; 2048];
    let result_p = PWSTR(result.as_mut_ptr());
    unsafe {
        FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, None, GetLastError().0, makelangid(LANG_NEUTRAL, SUBLANG_DEFAULT), result_p, 2048, None);
    }
    (&String::from_utf16(&result).unwrap() as &str).trim().to_string()
}

fn get_clock_tick() -> i64 {
    let mut dummy: i64 = 0;
    unsafe {
        let _ = QueryPerformanceCounter(&mut dummy);
    }
    dummy
}

fn get_clock_frequency() -> i64 {
    let mut dummy: i64 = 0;
    unsafe {
        let _ = QueryPerformanceFrequency(&mut dummy);
    }
    dummy
}

static mut G_INTERRUPT_REQUESTED: bool = false;
unsafe extern "system" fn ctrl_c_handler(signal: u32) -> BOOL{
    if signal == CTRL_C_EVENT {
        info("\nStoped", "Ctrl+C detected. Terminating child process", Status::SUCCESS);
        unsafe { G_INTERRUPT_REQUESTED = true; }
    }
    TRUE
}

pub fn execute_file(file_path: &str) -> (u32, f64) {
    let si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    unsafe {
        let _ = SetConsoleCtrlHandler(Some(ctrl_c_handler), true);
        let start_time = get_clock_tick();

        if CreateProcessW(
            None,
            PWSTR::from_raw(widestring::U16CString::from_str(file_path).unwrap().into_raw()),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0u32),
            None,
            None,
            &si,
            &mut pi,
        )
        .is_err()
        {
            info("\nFailed", &format!("cannot execute {}", file_path), Status::FAILURE);
            info("Error", &format!("{}: {}", GetLastError().0, get_error_message()), Status::FAILURE);
            pause_exit(Status::FAILURE);
        }

        while WaitForSingleObject(pi.hProcess, 1000) != WAIT_OBJECT_0 {
            if G_INTERRUPT_REQUESTED {
                let _ = TerminateProcess(pi.hProcess, Status::FAILURE as u32);
                break;
            }
        }

        let finish_time = get_clock_tick();

        let running_time = (finish_time - start_time) as f64 / get_clock_frequency() as f64;

        let mut result: u32 = 0;
        let _ = GetExitCodeProcess(pi.hProcess, &mut result);
        (result, running_time)
    }
}
