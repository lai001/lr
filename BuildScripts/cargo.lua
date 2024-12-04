task("cargo_upgrade_all")
do
    on_run(function()
        import("core.base.option")
        local black_list = {
            ["rs_computer_graphics"] = true
        }
        local is_incompatible = false
        if option.get("incompatible") then
            is_incompatible = true
        end
        for _, dir in ipairs(table.join(os.dirs("rs_*"), os.dirs("crates/rs_*"), os.dirs("programs/rs_*"))) do
            if black_list[dir] == nil then
                local old = os.cd(dir)
                if is_incompatible then
                    os.exec("cargo upgrade --incompatible")
                else
                    os.exec("cargo upgrade")
                end
                os.cd(old)
            end
        end
    end)
    set_menu {
        usage = "xmake cargo_upgrade_all",
        description = "Upgrade dependency version requirements in Cargo.toml manifest files.",
        options = {
            {'i', "incompatible", "k", nil, "Upgrade to latest incompatible version [default: ignore]"},
        }
    }
end
