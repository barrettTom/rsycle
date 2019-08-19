extern crate rsycle;

#[cfg(test)]
mod tests {
    use rsycle::main::{build_path, empty, list, restore, rsycle};
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Component, PathBuf};

    fn gen_rsyclebin(dir_name: &str) -> PathBuf {
        let mut rsyclebin = dirs::home_dir().unwrap();
        rsyclebin.push(dir_name);
        if !rsyclebin.exists() {
            fs::create_dir(&rsyclebin).unwrap();
        }
        rsyclebin
    }

    #[test]
    fn test_basics() {
        let filename = "test_file".as_ref();
        let test_path: PathBuf = [Component::CurDir, Component::Normal(filename)]
            .iter()
            .collect();
        assert!(!test_path.exists());
        fs::File::create(test_path.clone()).unwrap();

        let test_rsyclebin = gen_rsyclebin(".test_rsyclebin");
        test_rsycle(filename, test_rsyclebin.clone());
        test_restore(filename, test_rsyclebin.clone());
        test_cleanup(test_path, test_rsyclebin);
    }

    #[test]
    fn test_relative() {
        let filename = "../test_relative_file".as_ref();
        let test_path: PathBuf = [Component::CurDir, Component::Normal(filename)]
            .iter()
            .collect();
        assert!(!test_path.exists());
        fs::File::create(test_path.clone()).unwrap();

        let test_rsyclebin = gen_rsyclebin(".test_relative_rsyclebin");
        test_rsycle(filename, test_rsyclebin.clone());
        test_restore(filename, test_rsyclebin.clone());
        test_cleanup(test_path, test_rsyclebin);
    }

    fn test_rsycle(filename: &OsStr, rsyclebin: PathBuf) {
        let test_path = build_path(filename.to_str().unwrap()).unwrap();

        assert!(rsycle(rsyclebin.clone(), test_path.clone()).is_ok());

        assert!(!test_path.exists());

        assert!(list(rsyclebin.clone()).is_ok());
    }

    fn test_restore(filename: &OsStr, rsyclebin: PathBuf) {
        let test_path = build_path(filename.to_str().unwrap()).unwrap();

        assert!(restore(rsyclebin.clone(), test_path.clone()).is_ok());

        assert!(test_path.exists());
    }

    fn test_cleanup(path: PathBuf, rsyclebin: PathBuf) {
        assert!(empty(rsyclebin.clone()).is_ok());

        fs::remove_file(path).unwrap();
        fs::remove_dir_all(rsyclebin).unwrap();
    }
}
