using System.Reflection;
using System.Runtime.Loader;

namespace HotReload
{
    public class HotReloadAssemblyLoadContext : AssemblyLoadContext
    {
        private AssemblyDependencyResolver _resolver;

        public HotReloadAssemblyLoadContext(string path) : base(isCollectible: true)
        {
            _resolver = new AssemblyDependencyResolver(path);
        }

        protected override Assembly? Load(AssemblyName assemblyName)
        {
            Dictionary<string, Assembly> loadedAssemblies = new();
            foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
            {
                if (assembly.FullName != null)
                {
                    loadedAssemblies[assembly.FullName] = assembly;
                }
            }
            if (loadedAssemblies.ContainsKey(assemblyName.FullName))
            {
                return loadedAssemblies[assemblyName.FullName];
            }
            string? assemblyPath = _resolver.ResolveAssemblyToPath(assemblyName);
            if (assemblyPath != null)
            {
                return LoadFromAssemblyPath(assemblyPath);
            }
            return null;
        }

        protected override IntPtr LoadUnmanagedDll(string unmanagedDllName)
        {
            string? libraryPath = _resolver.ResolveUnmanagedDllToPath(unmanagedDllName);
            if (libraryPath != null)
            {
                return LoadUnmanagedDllFromPath(libraryPath);
            }
            return IntPtr.Zero;
        }
    }
}
