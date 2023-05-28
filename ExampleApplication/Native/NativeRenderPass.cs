using System.Runtime.InteropServices;

namespace Native
{
    using NativeRenderPassType = IntPtr;
    using NativeRenderPipelineType = IntPtr;

    public struct Range<T>
    {
        public T start;
        public T end;

        public Range(T start, T end)
        {
            this.start = start;
            this.end = end;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeRenderPassFunctions
    {
        public delegate* unmanaged<NativeRenderPassType, NativeRenderPipelineType, void> nativeRenderPassSetPipeline;
        public delegate* unmanaged<NativeRenderPassType, Range<uint>, Range<uint>, void> nativeRenderPassDraw;
        public delegate* unmanaged<NativeRenderPassType, void> nativeRenderPassDelete;
    }

    public unsafe class NativeRenderPass : IDisposable
    {
        public static NativeRenderPassFunctions Functions;

        private NativeRenderPassType nativeRenderPass = NativeRenderPassType.Zero;

        private bool disposed;

        public NativeRenderPass(NativeRenderPassType nativeRenderPass)
        {
            this.nativeRenderPass = nativeRenderPass;
        }

        public NativeRenderPassType GetNativeHandle()
        {
            return nativeRenderPass;
        }

        public void SetPipeline(NativeRenderPipeline nativeRenderPipeline)
        {
            Functions.nativeRenderPassSetPipeline(nativeRenderPass, nativeRenderPipeline.GetNativeHandle());
        }

        public void Draw(Range<uint> vertices, Range<uint> instances)
        {
            Functions.nativeRenderPassDraw(nativeRenderPass, vertices, instances);
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
                NativeRenderPass.Functions.nativeRenderPassDelete(nativeRenderPass);
                nativeRenderPass = NativeRenderPassType.Zero;
                disposed = true;
            }
        }

        ~NativeRenderPass()
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
