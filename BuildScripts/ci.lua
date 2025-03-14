local engine_root_dir = engine_root_dir
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

task("ci")
do
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.project.task")
        config.load()
        local extra_envs = {
            ["FFMPEG_DIR"] = ffmpeg_dir,
            ["RUSSIMP_PACKAGE_DIR"] = russimp_prebuild_dir
        }
        local tracy_android_libs_envs = {
            ["TRACY_CLIENT_LIB_PATH"] = path.join(engine_root_dir, "/build/android/arm64-v8a/debug"),
            ["TRACY_CLIENT_LIB"] = "tracy-client",
            ["TRACY_CLIENT_STATIC"] = 1
        }
        os.addenvs(extra_envs)
        os.cd(path.join(engine_root_dir, "rs_editor"))
        os.exec("cargo build --package rs_editor --bin editor --features editor --features renderdoc --features plugin_shared_crate --features plugin_dotnet --features plugin_v8")
        os.exec("cargo build --package rs_editor --bin editor --features editor --features renderdoc --features plugin_shared_crate --features plugin_dotnet --features plugin_v8 --release")
        os.cd(path.join(engine_root_dir, "build/target/release"))
        os.exec("./rs_shader_compiler.exe")
        os.cd(path.join(engine_root_dir, "rs_desktop_standalone"))
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone --features plugin_shared_crate --features standalone --features plugin_v8")
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone --features plugin_shared_crate --features standalone --features plugin_v8 --release")
        os.cd(engine_root_dir)
        os.addenvs(tracy_android_libs_envs)
        os.exec("xmake build_android_target --mode=debug --target=aarch64-linux-android")
        local envs = os.getenvs()
        envs["TRACY_CLIENT_LIB_PATH"] = path.join(engine_root_dir, "/build/android/arm64-v8a/release")
        os.setenvs(envs)
        os.exec("xmake build_android_target --mode=release --target=aarch64-linux-android")
    end)
    set_menu {
        usage = "xmake ci",
    }
end
