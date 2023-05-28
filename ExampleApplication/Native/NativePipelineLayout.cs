using System;
using System.Runtime.InteropServices;

namespace Native
{
    using NativePipelineLayoutType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativePipelineLayoutFunctions
    {
        public delegate* unmanaged<NativePipelineLayoutType, void> nativePipelineLayoutDelete;
    }

    public unsafe class NativePipelineLayout : IDisposable
    {
        private NativePipelineLayoutType nativePipelineLayout = NativePipelineLayoutType.Zero;

        public static NativePipelineLayoutFunctions? Functions;

        private bool disposed;

        public NativePipelineLayout(NativePipelineLayoutType nativePipelineLayout)
        {
            this.nativePipelineLayout = nativePipelineLayout;
        }

        public NativePipelineLayoutType GetNativeHandle()
        {
            return nativePipelineLayout;
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
                System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
                Functions.Value.nativePipelineLayoutDelete(nativePipelineLayout);
                nativePipelineLayout = NativePipelineLayoutType.Zero;
                disposed = true;
            }
        }

        ~NativePipelineLayout()
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
