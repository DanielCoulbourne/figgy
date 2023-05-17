use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write, Error, ErrorKind},
    fmt::Debug,
    path::Path,
};

#[derive(Debug)]
pub struct ConfigFile<T> {
    pub filename: String,
    pub directories: Vec<String>,
    pub absolute_filepath: Option<String>,
    pub default_config: Option<T>,
    pub create_if_missing: bool,
}

impl<'a, T: Serialize + Deserialize<'a> + Debug + Clone> ConfigFile<T> {
    pub fn new(filename: String) -> Self {
        Self {
            filename,
            directories: vec![],
            absolute_filepath: None,
            default_config: None,
            create_if_missing: false,
        }
    }

    pub fn directory(mut self, directory: String) -> Self {
        self.directories.push(directory);
        self
    }

    pub fn default(mut self, default: T) -> Self {
        self.default_config = Some(default);

        self
    }

    pub fn create_file_if_not_found(mut self) -> Self {
        self.create_if_missing = true;

        self
    }

    pub fn location(&mut self) -> Result<String, Error> {
        let mut dir: String = "".to_string();

        for directory in &self.directories {
            let filepath = format!("{}/{}", directory, self.filename);

            if Path::new(&filepath).exists() {
                dir = directory.to_string();
                break;
            }
        }

        if !self.directories.is_empty() && dir == *"" {
            if !self.create_if_missing {
                return Err(Error::new(ErrorKind::NotFound, "File was not found"));
            }

            dir = self.directories[0].to_string();
        }

        Ok(
            format!("{}/{}", dir, self.filename)
        )
    }

    pub fn get_config_from_default(self, path: Option<String>) -> Result<T, Error> {
        match self.default_config {
            Some(ref config) => {
                if path.is_some() {
                    let mut file = File::create(path.unwrap())?;
                    let config_json = serde_json::to_string_pretty(config)?;

                    file.write_all(config_json.as_bytes()).unwrap();
                }


                // @todo - Why do I have to clone this
                Ok(config.clone())
            },
            None => Err(Error::new(ErrorKind::NotFound, "No default config was provided"))
        }
    }

    pub fn get_file(&mut self) -> (Result<String, Error>, Result<File, Error>) {
        let filepath = self.location();

        match filepath {
            Ok(path) => {
                let file = File::open(&path);

                match file {
                    Ok(file_ok) => (Ok(path), Ok(file_ok)),
                    Err(error) => (Ok(path), Err(error)),
                }
            },
            Err(error) => {
                (
                    Err(error),
                    Err(Error::new(ErrorKind::NotFound, "Filename was invalid"))
                )
            }
        }
    }
    pub fn read_file(file: File) -> Result<T, Box<dyn std::error::Error>> {
        let mut contents = String::new();
        let mut file = file;

        file.read_to_string(&mut contents)?;

        let boxed_str = contents.into_boxed_str();
        let static_str = Box::leak(boxed_str);

        let config_data = serde_json::from_str(static_str)?;

        Ok(config_data)
    }
    

    pub fn get_config_from_file(self, file: File) -> Result<T, Error> {
        let file_contents = Self::read_file(file);

        match file_contents {
            Ok(config_data) => Ok(config_data),
            Err(_) => Err(Error::new(ErrorKind::NotFound, "File was invalid")),
        }
    }

    pub fn read(mut self) -> Result<T, Error> {
        let (filepath, file) = self.get_file();


        match file {
            Ok(file_ok) => self.get_config_from_file(file_ok),
            Err(_) => self.get_config_from_default(filepath.ok()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_file;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct PersonConfig {
        pub name: String,
        pub age: u8,
    }

    #[test]
    fn it_can_read_a_config_file() {
        let config = ConfigFile::<PersonConfig>::new("person.json".to_string())
            .directory("tests".to_string())
            .read();

        assert!(config.is_ok());
        
        let config = config.unwrap();

        assert_eq!(config.name, "Daniel");
        assert_eq!(config.age, 32);
    }

    #[test]
    fn it_uses_default_if_file_doesnt_exist () {
        let config = ConfigFile::<PersonConfig>::new("fake_file.json".to_string())
            .directory("tests".to_string())
            .default(PersonConfig {
                name: "Daniel".to_string(),
                age: 32,
            })
            .read();

        assert!(config.is_ok());
        
        let config = config.unwrap();

        assert_eq!(config.name, "Daniel");
        assert_eq!(config.age, 32);
    }

    #[test]
    fn it_can_write_the_defaults_to_a_file() {
        let config = ConfigFile::<PersonConfig>::new("create_file.json".to_string())
            .directory("tests".to_string())
            .create_file_if_not_found()
            .default(PersonConfig {
                name: "Daniel".to_string(),
                age: 32,
            })
            .read();

        assert!(config.is_ok());

        let created_file = File::open("tests/create_file.json");

        
        assert!(created_file.is_ok());

        let mut contents = String::new();
        created_file.unwrap().read_to_string(&mut contents).unwrap();

        assert_eq!(contents, "{\n  \"name\": \"Daniel\",\n  \"age\": 32\n}");

        remove_file(Path::new("tests/create_file.json")).unwrap();
    }
}