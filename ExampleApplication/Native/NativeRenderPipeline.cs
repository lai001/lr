using System.Runtime.InteropServices;

namespace Native
{
    using NativeRenderPipelineType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeRenderPipelineFunctions
    {
        public delegate* unmanaged<NativeRenderPipelineType, void> nativeRenderPipelineDelete;
    }

    public unsafe class NativeRenderPipeline : IDisposable
    {
        private NativeRenderPipelineType nativeRenderPipeline = NativeRenderPipelineType.Zero;

        public static NativeRenderPipelineFunctions? Functions;

        private bool disposed;

        public NativeRenderPipeline(NativeRenderPipelineType nativeRenderPipeline)
        {
            this.nativeRenderPipeline = nativeRenderPipeline;
        }

        public NativeRenderPipelineType GetNativeHandle()
        {
            return nativeRenderPipeline;
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
                Functions.Value.nativeRenderPipelineDelete(nativeRenderPipeline);
                nativeRenderPipeline = NativeRenderPipelineType.Zero;
                disposed = true;
            }
        }

        ~NativeRenderPipeline()
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
