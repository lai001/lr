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
        os.addenvs(extra_envs)
        os.cd(path.join(engine_root_dir, "rs_editor"))
        os.exec("cargo build --package rs_editor --bin rs_editor --features editor --features renderdoc")
        os.exec("cargo build --package rs_editor --bin rs_editor --features editor --features renderdoc --release")
        os.cd(path.join(engine_root_dir, "rs_desktop_standalone"))
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone --features standalone")
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone --features standalone --release")
        os.cd(engine_root_dir)
        os.exec("xmake build_android_target --mode=debug --target=aarch64-linux-android")
        os.exec("xmake build_android_target --mode=release --target=aarch64-linux-android")
    end)
    set_menu {
        usage = "xmake ci",
    }
end
