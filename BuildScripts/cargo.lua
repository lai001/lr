task("cargo_upgrade_all")
do
    on_run(function()
        import("core.base.option")
        local black_list = {
            ["rs_computer_graphics"] = true
        }
        local args = ""
        if option.get("incompatible") then
            args = args .. "--incompatible "
        end
        if option.get("offline") then
            args = args .. "--offline "
        end
        for _, dir in ipairs(table.join(os.dirs("rs_*"), os.dirs("crates/rs_*"), os.dirs("programs/rs_*"))) do
            if black_list[dir] == nil then
                local old = os.cd(dir)
                os.exec("cargo upgrade " .. args)
                os.cd(old)
            end
        end
    end)
    set_menu {
        usage = "xmake cargo_upgrade_all",
        description = "Upgrade dependency version requirements in Cargo.toml manifest files.",
        options = {
            {'i', "incompatible", "k", nil, "Upgrade to latest incompatible version [default: ignore]"},
            {'o', "offline", "k", nil, "Run without accessing the network"},
        }
    }
end

task("cargo_query_dep_crate_manifest_path")
    on_run(function()
        import("core.base.json")
        import("core.base.option")
        local dep_crate_name = nil
        local crate_rel_path = nil
        if option.get("crate") then
            crate_rel_path = option.get("crate")
        end
        if option.get("dep_crate") then
            dep_crate_name = option.get("dep_crate")
        end
        if dep_crate_name == nil or crate_rel_path == nil then
            raise("Usage: xmake cargo_query_dep_crate_manifest_path -d <dependent crate name> -c <crate relative path in engine>")
            return
        end
        local crate_abs_path = path.absolute(crate_rel_path)
        if os.isdir(crate_abs_path) == false then
            raise(format("%s is not a directory", crate_abs_path))
        end
        os.cd(crate_abs_path)
        local cmd = "cargo metadata --format-version=1"
        local outdata, errdata = os.iorun(cmd)
        local luatable = json.decode(outdata)
        local packages = luatable["packages"]
        for _, v in ipairs(packages)do
            if v["name"] == dep_crate_name then
                print(v["manifest_path"])
            end
        end
    end)
    set_menu {
        usage = "xmake cargo_query_dep_crate_manifest_path -d <dependent crate name> -c <crate relative path in engine>",
        description = "Query the local manifest path of the specified crate.",
        options = {
            {'c', "crate", "kv", nil, "The relative path of the crate inside the engine"},
            {'d', "dep_crate", "kv", nil, "The name of the dependent third-party crate"},
        }
    }