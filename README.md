# lr
## Build Requirements (Windows)
- Microsoft Visual Studio 2022 (MSVC 14.44.35207)
- Windows 10/11 with Windows SDK 10.0.22621.0
- xmake 3.0.8
- .net 8
- Rust 1.96.0
- clang 22.1.2 (Used for code formatting and generating Rust binding code and preprocessing shader code)
- Supported architectures: x64

## Build Requirements (Android)
- JDK 21
- build-tools 37
- ndk 25.1.8937393
- Supported architectures: arm64-v8a, x86_64

## Feature
- Hot reload

## Build
```
xmake setup
xmake ci
```

## Note
For android platform
```
xmake g --ndk=<path>
```
The above toolchain versions are the ones I used and tested on my local machine.  
Other developers may try compiling with lower versions of MSVC, Windows SDK, Rust, or other related tools, but compatibility is not guaranteed.