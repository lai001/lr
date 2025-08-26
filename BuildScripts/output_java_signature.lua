local engine_root_dir = engine_root_dir
task("generate_java_signature")
    on_run(function()
        import("lib.detect.find_program")
        local envs = os.getenvs()
        local java_home = envs["JAVA_HOME"]
        local search_paths = {}
        table.join2(search_paths, java_home)
        if java_home ~= nil then
            table.join2(search_paths, path.join(java_home, "bin"))
        end
        local javap_program = find_program("javap", {paths = search_paths, check = "-version"})
        local class_output_dir = path.join(engine_root_dir, "Android/Template/rs_android/build/intermediates/runtime_library_classes_dir/debug/com/lai001/rs_android")
        for _, file in ipairs(os.files(path.join(class_output_dir, "*.class"))) do
            local tmp_dir = path.join(engine_root_dir, "build/java/signature")
            if os.exists(tmp_dir) == false then
                os.mkdir(tmp_dir)
            end
            local basenaem = path.basename(file)
            local output_file = path.join(tmp_dir, basenaem .. ".signature")
            local cm = format("%s -s %s", javap_program, file)
            print(cm)
            local contents, error = os.iorun(cm)
            if error ~= nil and error ~= "" then
                print(error)
            end
            print(format("Write to %s", output_file))
            io.writefile(output_file, contents)
        end
    end)
    set_menu {
        usage = "xmake generate_java_signature",
        description = "Generate java signature",
        options = {
        }
    }

task("generate_android_java_signature")
    on_run(function()
        import("lib.detect.find_program")
        local envs = os.getenvs()
        local java_home = envs["JAVA_HOME"]
        local android_sdk_home = envs["ANDROID_SDK_HOME"]
        local search_paths = {}
        table.join2(search_paths, java_home)
        if java_home ~= nil then
            table.join2(search_paths, path.join(java_home, "bin"))
        end
        local javap_program = find_program("javap", {paths = search_paths, check = "-version"})
        local android_jar_path = path.join(android_sdk_home, "platforms/android-34/android.jar")
        local classes = {
            "java.io.InputStream",
            "android.view.Surface",
            "android.view.MotionEvent",
            "android.view.KeyEvent"
        }
        for _, class in ipairs(classes) do
            local tmp_dir = path.join(engine_root_dir, "build/java/signature/android")
            if os.exists(tmp_dir) == false then
                os.mkdir(tmp_dir)
            end
            local cm = format("%s -s -cp %s %s", javap_program, android_jar_path, class)
            local file_name = format("%s.signature", class)
            local output_file = path.join(tmp_dir, file_name)
            print(cm)
            local contents, error = os.iorun(cm)
            if error ~= nil and error ~= "" then
                print(error)
            end
            print(format("Write to %s", output_file))
            io.writefile(output_file, contents)
        end
    end)
    set_menu {
        usage = "xmake generate_android_java_signature",
        description = "Generate android java signature",
        options = {
        }
    }