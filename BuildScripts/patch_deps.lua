local engine_root_dir = engine_root_dir

local function fix_timezone_provider(path, os, json, semver, print, crate_path)
    crate_path = path.join(engine_root_dir, crate_path)
    print("Checking " .. crate_path)
    os.cd(crate_path)
    local is_plugin_v8_feature_exist = json.decode(os.iorun("cargo read-manifest"))["features"]["plugin_v8"] ~= nil
    local features = ""
    if is_plugin_v8_feature_exist then
        features = "--features plugin_v8"
    end
    local outdata, errdata = os.iorun("cargo metadata --format-version 1 " .. features)
    local luatable = json.decode(outdata)
    for _, package in ipairs(luatable["packages"]) do
        local package_name = package["name"]
        local package_version = package["version"]
        if package_name == "timezone_provider" then
            local version_comparison = semver.compare(package_version, "0.0.16")
            if version_comparison == 0 then
                os.exec("cargo update timezone_provider --precise 0.0.14")
            else
                print("Skip")
            end
        end
    end
end

task("patch_deps")
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.base.semver")
        config.load()
        fix_timezone_provider(path, os, json, semver, print, "rs_editor")
        fix_timezone_provider(path, os, json, semver, print, "rs_desktop_standalone")
        fix_timezone_provider(path, os, json, semver, print, "rs_v8_host")
    end)
    set_menu {
        usage = "xmake patch_deps",
        description = "Patch dependencies",
        options = {
        }
    }
