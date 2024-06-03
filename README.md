# rs_computer_graphics

Windows 10  
Android     
.net6   
Rust 1.78.0 

## Feature
- Hot reload

## Build
```
xmake download_deps
xmake build_3rdparty
xmake setup
xmake gen_config
xmake ci
```

## Note
For android platform
```
xmake g --ndk=<path>
```