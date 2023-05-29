# figgy
A small Rust library for managing config files which supports heirarchical directories, defaults, and writing initial files

## Example:
_~/.config/myapp/myapp.config.json_
OR
_~/.myapp/myapp.config.json_
```json
{
    "api_key": "1234abcdef!@#$%",
    "api_version": 3
}
```

Your Rust program:
```rust
    use figgy::ConfigFile;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct ApiKeysConfig {
        api_key: String,
        api_version: i16,
    }

    let config = ConfigFile::<ApiKeysConfig>::new("myapp.config.json")
            .directory("~/.config/myapp/")
            .directory("~/.myapp/")
            .read();
```

## Default Configs
If you want a default configuration, you can set one with `.default()`.
If you want to automatically write a config file if none is detected, use `.create_file_if_not_found()`

```rust
let config = ConfigFile::<ApiKeysConfig>::new("myapp.config.json")))
            .directory("~/.config/myapp/")
            .directory("~/.myapp/")
            .create_file_if_not_found()
            .default(ApiKeysConfig {
                api_key: "Super secret API key",
                api_version: 25,
            })
            .read();
```
