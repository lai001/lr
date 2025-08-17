local ffmpeg_dir = ffmpeg_dir
local deps_dir = deps_dir

task("gen_config")
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        config.load()
        local is_dry = false
        if option.get("dry") ~= nil then
            is_dry = true
        end
        local ndk_path = (get_config("ndk") and { get_config("ndk") } or { os.getenv("NDK_HOME") })[1]
        ndk_path = (ndk_path and { ndk_path } or { os.getenv("NDK_ROOT") })[1]
        if ndk_path == nil then
            os.raise("NDK not found")
        end

        local target_template = [[
[target.@target@]
ar = "@ndk@/toolchains/llvm/prebuilt/@host@-x86_64/bin/llvm-ar.exe"
linker = "@ndk@/toolchains/llvm/prebuilt/@host@-x86_64/bin/clang.exe"
rustflags = ["-Clink-args=--target=@target@@api@"]
        ]]

        local content = ""
        local targets = { "aarch64-linux-android", "arm-linux-androideabi", "armv7-linux-androideabi",
            "x86_64-linux-android", "i686-linux-android" }

        for _, target in pairs(targets) do
            local t = target_template:gsub("@target@", target)
            t = t:gsub("@api@", "30")
            t = t:gsub("@ndk@", ndk_path)
            -- t = t:gsub("@host@", get_config("plat"))
            t = t:gsub("@host@", "windows")
            content = content .. "\n" .. t
        end
        content = content .. "\n" .. [[
[build]
# rustflags = ["-C", "prefer-dynamic", "-C", "rpath"]
rustflags = ["-C", "rpath"]
target-dir = "./build/target"
        ]]
        local ffmpeg_block = [[
[env]
FFMPEG_DIR = "%s"

[target.aarch64-linux-android.env]
FFMPEG_DIR = "%s"

[target.x86_64-linux-android.env]
FFMPEG_DIR = "%s"
        ]]
        ffmpeg_block = format(ffmpeg_block, ffmpeg_dir, path.join(deps_dir, "ffmpeg_android/arm64-v8a"), path.join(deps_dir, "ffmpeg_android/x86_64"))
        local fix_conflict_library = [[
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=/FORCE:MULTIPLE"]
        ]]
        content = content .. "\n" .. ffmpeg_block
        content = content .. "\n" .. fix_conflict_library
        content = content:gsub("\\", "/")
        if is_dry then
            print(content)
        else
            io.writefile(".cargo/config.toml", content)
        end
    end)
    set_menu {
        usage = "xmake gen_config",
        description = "Generate cargo config.toml",
        options = {
            { "d", "dry", "k", nil, nil },
        }
    }
