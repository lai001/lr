# lr

Windows 10
Android
.net8
Rust 1.78.0
xmake
clang（optional）

## Feature
- Hot reload

## Build
```
xmake download_deps
xmake build_3rdparty
xmake compile_tool
xmake setup
xmake gen_config
xmake ci
```

## Note
For android platform
```
xmake g --ndk=<path>
```