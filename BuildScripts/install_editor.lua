local engine_root_dir = engine_root_dir
task("install_editor") do
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.project.task")
        config.load()

        local prefix = option.get("prefix")
        if prefix == nil then
            prefix = path.join(engine_root_dir, "build/Editor")
        end

        if os.exists(prefix) == false then
            print(format("Create %s folder.", prefix))
            os.mkdir(prefix)
        end
        os.cp(path.join(engine_root_dir, "rs_editor/target/*/rs_editor.exe"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "rs_editor/target/*/*.dll"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "Resource/Editor"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "Resource/Remote/Font"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "rs_computer_graphics/src/shader/attachment.wgsl"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "rs_render/shaders/*.wgsl"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "rs_editor/target/shaders/*.wgsl"), path.join(prefix, "rs_render/shaders"))
        os.cp(path.join(engine_root_dir, "rs_editor/target/shaders/*.wgsl"), path.join(prefix, "rs_computer_graphics/src/shader"))
        os.cp(path.join(engine_root_dir, "rs_desktop_standalone/target/*/rs_desktop_standalone.exe"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "rs_desktop_standalone/target/*/*.dll"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "rs_media_cmd/target/*/rs_media_cmd.exe"), prefix, {rootdir = engine_root_dir})
        os.cp(path.join(engine_root_dir, "ExampleApplication"), prefix, {rootdir = engine_root_dir})
    end)
    set_menu {
        usage = "xmake install_editor",
        description = "Install Editor",
        options = {
            { "p", "prefix", "kv", nil, "Set install prefix." },
        }
    }
end