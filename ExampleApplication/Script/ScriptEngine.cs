using Foundation;
using System;
using System.IO;
using System.Linq;
using HotReload;
using Native;

namespace Script
{
    public class UserSscriptPayload
    {
        private WeakReference userScriptWeakRef;

        public UserSscriptPayload(IUserScript userScript)
        {
            userScriptWeakRef = new WeakReference(userScript);
        }

        public void Initialize()
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.Initialize();
            }
        }

        public void Tick(NativeEngine engine)
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.Tick(engine);
            }
        }

        public void KeyboardInput(NativeKeyboardInput keyboardInput)
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.KeyboardInput(keyboardInput);
            }
        }

        public void CursorMoved(PhysicalPosition physicalPosition)
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.CursorMoved(physicalPosition);
            }
        }
    }

    public class ScriptEngine
    {
        public UserSscriptPayload userSscriptPayload;

        public ScriptEngine()
        {
        }

        //public void Reload()
        //{
        //    string fileName = "UserScript.dll";
        //    RemoteAssembly.SetAssemblyPath($"./{fileName}");
        //    string source = Path.Join(Directory.GetCurrentDirectory(), "tmp", fileName);
        //    string target = Path.Join(Directory.GetCurrentDirectory(), fileName);
        //    try
        //    {
        //        RemoteAssembly.UnloadAssembly();
        //        File.Copy(source, target, true);
        //        Console.WriteLine($"copy {source} to {target}.");
        //        RemoteAssembly.LoadAssembly();
        //        userSscript = new UserSscript(Util.CreateInstances<IUserScript>(RemoteAssembly.GetAssembly()).First());
        //    }
        //    catch (Exception ex)
        //    {
        //        Console.WriteLine($"{ex}");
        //    }
        //}

        public void Reload(string Path)
        {
            RemoteAssembly.SetAssemblyPath(Path);
            try
            {
                RemoteAssembly.UnloadAssembly();
                RemoteAssembly.LoadAssembly();
                userSscriptPayload = new UserSscriptPayload(Util.CreateInstances<IUserScript>(RemoteAssembly.GetAssembly()).First());
            }
            catch (Exception ex)
            {
                Console.WriteLine($"{ex}");
            }
        }
    }
}
