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
            {'o', "offline", "k", nil, "Upgrade to latest incompatible version [default: ignore]"},
        }
    }
end
