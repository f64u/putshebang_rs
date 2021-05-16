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

        let shebang = if possible_shebang.starts_with("#!") {
            Some(possible_shebang)
        } else {
            None
        };

        let mode = fs::metadata(path)?.permissions().mode();

        let mut contents = String::new();
        rdr.read_to_string(&mut contents)?;

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

fn main() {
    let mut sfile = SFile::new(Path::new("/home/fadal/hello.py")).expect("Fucking erred.");

    println!("{:?}", sfile.shebang());
    println!("{:?}", sfile.contents());
    sfile.make_executable().expect("Making exec fucking erred.");
    println!("{:?}", sfile.executable());
}
