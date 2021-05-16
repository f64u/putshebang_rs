use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::{
    fs::{self, File},
    os::unix::prelude::PermissionsExt,
};

struct SFile<'a> {
    _descriptor: File,
    path: &'a Path,
    shebang: Option<String>,
    executable: bool,
    contents: String,
}

impl<'a> SFile<'a> {
    fn new(path: &'a Path) -> Result<Self, io::Error> {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(&path)?;

        let mut rdr = BufReader::new(&f);
        let mut possible_shebang = String::new();
        rdr.read_line(&mut possible_shebang)?;
        let mut rest = String::new();
        rdr.read_to_string(&mut rest)?;

        let mut contents = String::new();

        let shebang = if possible_shebang.starts_with("#!") {
            Some(possible_shebang)
        } else {
            contents.push_str(possible_shebang.as_str());
            None
        };
        contents.push_str(rest.as_str());

        let mode = fs::metadata(path)?.permissions().mode();

        Ok(Self {
            _descriptor: f,
            path,
            shebang: shebang,
            executable: (mode & 0o111) != 0,
            contents: contents,
        })
    }

    fn shebang(&self) -> &Option<String> {
        &self.shebang
    }

    fn executable(&self) -> bool {
        self.executable
    }

    fn contents(&self) -> &String {
        &self.contents
    }

    /// Change the mode of the file to be executable.
    /// Example:
    ///     -rw-rw-rw- becomes -rwxrwxrwx
    ///     -rw------ becomes -rwx------
    fn make_executable(&mut self) -> Result<(), io::Error> {
        if !self.executable {
            let mut perms = fs::metadata(self.path)?.permissions();
            let mut mode = perms.mode();
            mode |= (mode & 0o444) >> 2;
            perms.set_mode(mode);
            fs::set_permissions(self.path, perms)?;
            self.executable = true;
        }
        Ok(())
    }

    fn write(&mut self) {}
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn sfile_picks_up_shebang_contents_and_executability() -> Result<(), std::io::Error> {
        let tmpfile = NamedTempFile::new()?;
        write!(
            tmpfile.as_file(),
            "#!/usr/bin/python\n\nprint(\"Hello, World!\")"
        )?;

        let tmppath = tmpfile.into_temp_path();

        let sfile = SFile::new(&tmppath)?;
        assert_eq!(sfile.shebang(), &Some(String::from("#!/usr/bin/python\n")));
        assert_eq!(sfile.executable(), false);
        assert_eq!(
            sfile.contents(),
            &String::from("\nprint(\"Hello, World!\")")
        );

        tmppath.close()?;
        Ok(())
    }

    #[test]
    fn sfile_no_shebang() -> Result<(), std::io::Error> {
        let tmpfile = NamedTempFile::new()?;
        write!(
            tmpfile.as_file(),
            "This is a regular file.\n\nNo shebang is here.\n"
        )?;

        let tmppath = tmpfile.into_temp_path();

        let sfile = SFile::new(&tmppath)?;
        assert_eq!(sfile.shebang(), &None);
        assert_eq!(sfile.executable(), false);
        assert_eq!(
            sfile.contents(),
            &String::from("This is a regular file.\n\nNo shebang is here.\n")
        );

        tmppath.close()?;
        Ok(())
    }

    #[test]
    fn make_shabang_works() -> Result<(), std::io::Error> {
        let tmpfile = NamedTempFile::new()?;
        write!(
            tmpfile.as_file(),
            "#!/usr/bin/python\n\nprint(\"Hello, World!\")"
        )?;

        let tmppath = tmpfile.into_temp_path();

        let mut sfile = SFile::new(&tmppath)?;
        assert_eq!(sfile.executable(), false);
        sfile.make_executable()?;
        assert_eq!(sfile.executable(), true);

        tmppath.close()?;
        Ok(())
    }
}
