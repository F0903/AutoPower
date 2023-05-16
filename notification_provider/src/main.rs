mod toast;

use autopower_shared::logging::Logger;
use autopower_shared::notifications::{NOTIFICATION_PIPE_NAME, PIPE_PATH_ROOT};
use autopower_shared::util::to_win32_wstr;
use autopower_shared::{notifications::NotificationCommand, util::get_last_win32_err};
use toast::Toast;
use windows::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND},
    System::{
        Com::CoInitialize,
        Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_TYPE_MESSAGE},
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const PIPE_INPUT_BUFFER_SIZE: u32 = 512;

const LOGGER: Logger = Logger::new("main", "autopower_notifier");

fn execute_display_command(command: NotificationCommand) -> Result<()> {
    let mut cmd_lines = command.content.lines();
    let title = cmd_lines.next().expect("Could not get next cmd line!");
    let content = cmd_lines
        .next()
        .expect("Could not get next second cmd line!");
    let toast = Toast::new(title, content);
    toast.send()?;
    Ok(())
}

fn execute_command(command: NotificationCommand) -> Result<()> {
    match command.name.as_str() {
        "display" => execute_display_command(command),
        _ => Ok(()),
    }
}

fn read_notification_command(input_handle: HANDLE) -> Result<NotificationCommand> {
    let mut buf: [u8; PIPE_INPUT_BUFFER_SIZE as usize] = [0; PIPE_INPUT_BUFFER_SIZE as usize];
    let mut count: u32 = 0;
    let result = unsafe {
        ReadFile(
            input_handle,
            Some(buf.as_mut_ptr() as *mut std::ffi::c_void),
            PIPE_INPUT_BUFFER_SIZE,
            Some(&mut count as *mut u32),
            None,
        )
    };
    if !result.as_bool() {
        let err = get_last_win32_err()?;
        let err_msg = format!("Could not read from pipe!\n{}", err);
        LOGGER.debug_log(&err_msg);
        return Err(err_msg.into());
    }

    let str = std::str::from_utf8(&buf[..count as usize])?;

    LOGGER.debug_log(format!("notification_provider: read input:\n{}", str));
    let object = serde_json::from_str::<NotificationCommand>(str)?;
    Ok(object)
}

fn create_pipe() -> Result<HANDLE> {
    let pipe_name = to_win32_wstr(&format!("{}{}", PIPE_PATH_ROOT, NOTIFICATION_PIPE_NAME));
    let pipe = unsafe {
        LOGGER.debug_log("Creating pipe...");
        let pipe = CreateNamedPipeW(
            pipe_name.get_const(),
            PIPE_ACCESS_INBOUND,
            PIPE_TYPE_MESSAGE,
            1,
            0,
            PIPE_INPUT_BUFFER_SIZE,
            0,
            None,
        );
        if pipe.is_invalid() {
            let err = get_last_win32_err()?;
            LOGGER.debug_log(format!("Pipe is invalid!\n{}", err));
        }
        LOGGER.debug_log("Waiting for pipe connection...");
        if !ConnectNamedPipe(pipe, None).as_bool() {
            let err = get_last_win32_err()?;
            LOGGER.debug_log(format!(
                "Could not wait for client to connect to pipe!\n{}",
                err
            ));
        }
        pipe
    };
    Ok(pipe)
}

fn wait_for_input() -> Result<()> {
    let pipe = create_pipe()?;
    LOGGER.debug_log("notification_provider: waiting for input...");
    loop {
        let command = read_notification_command(pipe)?;
        execute_command(command)?;
    }
}

fn run() -> Result<()> {
    unsafe { CoInitialize(None)? };
    wait_for_input()?;
    Ok(())
}

fn main() -> Result<()> {
    match run() {
        Ok(_) => (),
        Err(e) => {
            LOGGER.debug_log(format!("Exited with error!\n{}", e));
        }
    }
    Ok(())
}
