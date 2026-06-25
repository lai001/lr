local deps_dir = deps_dir
local quickjs_dir = quickjs_dir
local gizmo_dir = gizmo_dir
local ffmpeg_dir = ffmpeg_dir
local metis_dir = metis_dir
local gklib_dir = gklib_dir
local russimp_prebuild_dir = russimp_prebuild_dir
local tracy_root_dir = tracy_root_dir
local dotnet_sdk_dir = dotnet_sdk_dir
local check_hash_files = check_hash_files
local hash_files = hash_files
local kcp_root_dir = kcp_root_dir
local assimp_root_dir = assimp_root_dir

task("hash_files")
do
    on_run(function()
        import("core.base.option")
        import("core.base.bytes")
        local input = option.get("input")
        local is_trace = option.get("trace") ~= nil
        if input == nil then
            raise()
        end
        local value = hash_files(os, io, bytes, is_trace, input)
        print(value)
    end)
    set_menu {
        usage = "xmake hash_files",
        description = "Hash files",
        options = {
            { "i", "input", "kv", nil, "File path or pattern match" },
            { "d", "trace", nil, nil, "Enable trace mode" },
        }
    }
end

task("download_deps")
do
    on_run(function()
        import("net.http")
        import("utils.archive")
        import("devel.git")
        import("core.project.config")
        import("core.base.bytes")
        config.load()
        os.mkdir(deps_dir)

        local is_enable_dotnet = get_config("enable_dotnet")
        local is_enable_quickjs = get_config("enable_quickjs")

        if is_enable_dotnet then
            local dotnetSDKFilename = "dotnet-sdk-8.0.302-win-x64.zip"
            local link =
                "https://download.visualstudio.microsoft.com/download/pr/5af098e1-e433-4fda-84af-3f54fd27c108/6bd1c6e48e64e64871957289023ca590/" ..
                dotnetSDKFilename

            if not check_hash_files(os, io, bytes, path.join(deps_dir, dotnetSDKFilename), "56d7cd50cb936ef098039e93bef6656e") then
                print("Download dotnet")
                http.download(link, path.join(deps_dir, dotnetSDKFilename))
            end
            if not check_hash_files(os, io, bytes, dotnet_sdk_dir .. "/**/*.exe", "f98dc7bb0b9c6462975c7ad0bc6b55b7") then
                print(format("Extract dotnet to %s", dotnet_sdk_dir))
                archive.extract(path.join(deps_dir, dotnetSDKFilename), dotnet_sdk_dir)
            end
        end

        if is_enable_quickjs then
            if os.exists(quickjs_dir) == false then
                if is_plat("windows") then
                    git.clone("https://github.com/c-smile/quickjspp.git", { outputdir = quickjs_dir })
                else
                    git.clone("https://github.com/bellard/quickjs.git", { outputdir = quickjs_dir })
                end
                git.checkout("master", { repodir = quickjs_dir })
            end
        end

        if os.exists(gizmo_dir) == false then
            git.clone("https://github.com/urholaukkarinen/transform-gizmo.git", { outputdir = gizmo_dir })
            git.checkout("85e09401ee3b827668c5ad7cc9160aa9e544207a", { repodir = gizmo_dir })
        end

        if not check_hash_files(os, io, bytes, "Resource/Remote/neon_photostudio_2k.exr", "3ad9df2cce80d8737e2a51be73a6507d") then
            local link = "https://dl.polyhaven.org/file/ph-assets/HDRIs/exr/2k/neon_photostudio_2k.exr"
            print("Download neon_photostudio_2k.exr")
            http.download(link, "Resource/Remote/neon_photostudio_2k.exr")
        end

        local ffmpeg_zip_filename = path.join(deps_dir, "ffmpeg-n7.1.4-win64-gpl-shared-7.1.zip")
        if not check_hash_files(os, io, bytes, ffmpeg_zip_filename, "a931f06f18583bae37d5f1dfd00336b1") then
            local link =
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-05-15-13-34/ffmpeg-n7.1.4-win64-gpl-shared-7.1.zip"
            print("Download ffmpeg")
            http.download(link, ffmpeg_zip_filename)
        end
        if not check_hash_files(os, io, bytes, ffmpeg_dir .. "/**/*.dll", "389d5abb6480b4f10e6e95c046cb674a") then
            print(format("Extract ffmpeg to %s", deps_dir))
            archive.extract(ffmpeg_zip_filename, deps_dir)
        end

        if not check_hash_files(os, io, bytes, "Resource/Remote/xgplayer-demo-360p.mp4", "2b8918b9a0f8bfbfae7b2775c60118d1") then
            local link = "https://sf1-cdn-tos.huoshanstatic.com/obj/media-fe/xgplayer_doc_video/mp4/xgplayer-demo-360p.mp4"
            print("Download xgplayer-demo-360p.mp4")
            http.download(link, "Resource/Remote/xgplayer-demo-360p.mp4")
        end

        if not check_hash_files(os, io, bytes, "Resource/Remote/sample-15s.mp3", "b402473130c79fdc8ec88f5f244fc796") then
            local link = "https://download.samplelib.com/mp3/sample-15s.mp3"
            http.download(link, "Resource/Remote/sample-15s.mp3")
        end

        local meshopt_rs_dir = path.join(deps_dir, "meshopt-rs")
        if os.exists(meshopt_rs_dir) == false then
            git.clone("https://github.com/gwihlidal/meshopt-rs.git", { outputdir = meshopt_rs_dir })
            git.checkout("master", { repodir = meshopt_rs_dir })
        end

        if os.exists(metis_dir) == false then
            git.clone("https://github.com/KarypisLab/METIS.git", { outputdir = metis_dir })
            git.checkout("v5.2.1", { repodir = metis_dir })
        end

        if os.exists(gklib_dir) == false then
            git.clone("https://github.com/KarypisLab/GKlib.git", { outputdir = gklib_dir })
            git.checkout("master", { repodir = gklib_dir })
        end

        local russimp_file = path.join(deps_dir, "russimp-2.0.2-x86_64-pc-windows-msvc-static.tar.gz")
        if os.exists(russimp_file) == false then
            local link = "https://github.com/jkvargas/russimp-sys/releases/download/v2.0.2/russimp-2.0.2-x86_64-pc-windows-msvc-static.tar.gz"
            http.download(link, russimp_file)
        end
        -- if os.exists(russimp_prebuild_dir) == false and os.exists(russimp_file) then
        --     archive.extract(russimp_file, russimp_prebuild_dir)
        -- end

        local tracy_archive_file = path.join(deps_dir, "Tracy-0.13.1.zip")
        local tracy_file = path.join(deps_dir, "Tracy-0.13.1")
        if os.exists(tracy_archive_file) == false then
            local link = "https://github.com/wolfpld/tracy/releases/download/v0.13.1/windows-0.13.1.zip"
            http.download(link, tracy_archive_file)
        end
        if os.exists(tracy_file) == false and os.exists(tracy_archive_file) then
            archive.extract(tracy_archive_file, tracy_file)
        end

        if os.exists(tracy_root_dir) == false then
            git.clone("https://github.com/wolfpld/tracy.git", { outputdir = tracy_root_dir })
            git.checkout("v0.13.1", { repodir = tracy_root_dir })
        end

        if os.exists(kcp_root_dir) == false then
            git.clone("https://github.com/skywind3000/kcp.git", { outputdir = kcp_root_dir })
            git.checkout("f4f3a89cc632647dabdcb146932d2afd5591e62e", { repodir = kcp_root_dir })
        end

        local ktx_software_file = path.join(deps_dir, "KTX-Software-4.4.2-Windows-x64.zip")
        local ktx_software = path.join(deps_dir, "KTX-Software-4.4.2-Windows-x64")
        if os.exists(ktx_software_file) == false then
            local link = "https://github.com/KhronosGroup/KTX-Software/releases/download/v4.4.2/KTX-Software-4.4.2-Windows-x64.exe"
            print(format("Download %s", link))
            http.download(link, ktx_software_file)
        end
        if os.exists(ktx_software) == false and os.exists(ktx_software_file) then
            print(format("Extract %s to %s", ktx_software_file, ktx_software))
            archive.extract(ktx_software_file, ktx_software)
        end

        local compressonatorcli_file = path.join(deps_dir, "compressonatorcli-4.5.52-win64.zip")
        local compressonatorcli = path.join(deps_dir, "compressonatorcli-4.5.52-win64")
        if os.exists(compressonatorcli_file) == false then
            local link = "https://github.com/GPUOpen-Tools/compressonator/releases/download/V4.5.52/compressonatorcli-4.5.52-win64.zip"
            print(format("Download %s", link))
            http.download(link, compressonatorcli_file)
        end
        if os.exists(compressonatorcli) == false and os.exists(compressonatorcli_file) then
            print(format("Extract %s to %s", compressonatorcli_file, compressonatorcli))
            archive.extract(compressonatorcli_file, compressonatorcli)
        end

        if os.exists(assimp_root_dir) == false then
            git.clone("https://github.com/assimp/assimp.git", { outputdir = assimp_root_dir })
            git.checkout("v6.0.5", { repodir = assimp_root_dir })
        end
    end)
    set_menu {
        usage = "xmake download_deps",
        description = "Download dependencies.",
        options = {
            { nil, "download_deps", nil, nil, nil },
        }
    }
end
