task("gen_config")
do
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        config.load()

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
            t = t:gsub("@host@", get_config("plat"))
            content = content .. "\n" .. t
        end
        io.writefile(".cargo/config.toml", content)
    end)
    set_menu {
        usage = "xmake gen_config",
        description = "Generate cargo config.toml",
        options = {
            { nil, "gen_config", nil, nil, nil },
        }
    }
end
