local csharp_workspace_name = csharp_workspace_name
task("build_clean")
do
    on_run(function()
        for _, dir in ipairs(os.dirs("rs_*/target")) do
            os.tryrm(dir)
        end
        for _, dir in ipairs(os.dirs("Android/Template/*/build")) do
            os.tryrm(dir)
        end
        os.tryrm(csharp_workspace_name .. "/.vs")
        os.tryrm("build")
        for _, dir in ipairs(os.dirs(csharp_workspace_name .. "/**/obj")) do
            os.tryrm(dir)
        end
        for _, dir in ipairs(os.files("Android/Template/rs_android/src/main/jniLibs/*/*.so")) do
            os.tryrm(dir)
        end        
    end)
    set_menu {
        usage = "xmake build_clean",
        description = "Clean up build files.",
        options = {
            { nil, "build_clean", nil, nil, nil },
        }
    }
end
