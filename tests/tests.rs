extern crate rsycle;

#[cfg(test)]
mod tests {
    use rsycle::main::{build_path, empty, list, restore, rsycle};
    use std::fs;
    use std::path::{Component, PathBuf};

    fn get_rsyclebin() -> PathBuf {
        let mut rsyclebin = dirs::home_dir().unwrap();
        rsyclebin.push(".test_rsyclebin");
        if !rsyclebin.exists() {
            fs::create_dir(&rsyclebin).unwrap();
        }
        rsyclebin
    }

    #[test]
    fn test_rsycle() {
        let rsyclebin = get_rsyclebin();

        let filename = "test_file".as_ref();

        let mut test_path: PathBuf = [Component::CurDir, Component::Normal(filename)]
            .iter()
            .collect();

        assert!(!test_path.exists());

        fs::File::create(test_path.clone()).unwrap();

        test_path = build_path(filename.to_str().unwrap()).unwrap();

        assert!(rsycle(rsyclebin.clone(), test_path.clone()).is_ok());

        assert!(!test_path.exists());

        assert!(list(rsyclebin.clone()).is_ok());

        assert!(restore(rsyclebin.clone(), test_path.clone()).is_ok());

        assert!(test_path.exists());

        assert!(empty(rsyclebin.clone()).is_ok());

        fs::remove_file(test_path).unwrap();
        fs::remove_dir_all(rsyclebin).unwrap();
    }
}
