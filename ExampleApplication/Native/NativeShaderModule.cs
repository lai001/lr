using System.Runtime.InteropServices;

namespace Native
{
    using NativeShaderModuleType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeShaderModuleFunctions
    {
        public delegate* unmanaged<NativeShaderModuleType, void> nativeShaderModuleDelete;
    }

    public unsafe class NativeShaderModule : IDisposable
    {
        private NativeShaderModuleType nativeShaderModule = NativeShaderModuleType.Zero;

        public static NativeShaderModuleFunctions? Functions;

        private bool disposed;

        public NativeShaderModule(NativeShaderModuleType nativeShaderModule)
        {
            this.nativeShaderModule = nativeShaderModule;
        }

        public NativeShaderModuleType GetNativeHandle()
        {
            return nativeShaderModule;
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!disposed)
            {
                if (disposing)
                {
                    // Manual release of managed resources.
                }

                // Release unmanaged resources.
                Console.WriteLine($"NativeShaderModule destructed.");
                System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
                Functions.Value.nativeShaderModuleDelete(nativeShaderModule);
                nativeShaderModule = NativeShaderModuleType.Zero;
                disposed = true;
            }
        }

        ~NativeShaderModule()
        {
            Dispose(disposing: false);
        }

        public void Dispose()
        {
            Dispose(disposing: true);
            GC.SuppressFinalize(this);
        }
    }
}
