type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn copy_powershell_scripts() -> Result<()> {
    for file in std::fs::read_dir("./")? {
        let file = file?;
        let file_name = file.file_name();
        let file_name_str = file_name
            .to_str()
            .ok_or("Could not get string from osstring")?;
        let file_extension = match file_name_str.rsplit_once('.') {
            Some(x) => x.1,
            None => continue,
        };
        if file_extension != "ps1" {
            continue;
        }

        std::fs::copy(file.path(), format!("./target/release/{}", file_name_str))?;
    }
    Ok(())
}

fn main() -> Result<()> {
    copy_powershell_scripts()?;
    Ok(())
}
