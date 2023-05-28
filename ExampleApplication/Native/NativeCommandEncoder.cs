using System.Runtime.InteropServices;

namespace Native
{
    using NativeRenderPassType = IntPtr;
    using NativeCommandEncoderType = IntPtr;
    using NativeTextureViewType = IntPtr;
    using NativeCommandBufferType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeCommandEncoderFunctions
    {
        public unsafe delegate* unmanaged<NativeCommandEncoderType, NativeTextureViewType, NativeRenderPassType> nativeCommandEncoderBeginRenderPass;
        public unsafe delegate* unmanaged<NativeCommandEncoderType, NativeCommandBufferType> nativeCommandEncoderFinish;
    }

    public unsafe struct NativeCommandEncoder
    {
        public static NativeCommandEncoderFunctions? Functions;

        private NativeCommandEncoderType nativeCommandEncoder = NativeCommandEncoderType.Zero;

        private bool isVaild = true;

        public NativeCommandEncoder(NativeCommandEncoderType nativeCommandEncoder)
        {
            this.nativeCommandEncoder = nativeCommandEncoder;
        }

        public NativeCommandEncoderType GetNativeHandle()
        {
            if (!isVaild)
            {
                return NativeCommandEncoderType.Zero; 
            }
            return nativeCommandEncoder;
        }

        public NativeRenderPass BeginRenderPass(NativeTextureView nativeTextureView)
        {
            if (!isVaild)
            {
                throw new Exception("");
            }
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            IntPtr nativeRenderPass = Functions.Value.nativeCommandEncoderBeginRenderPass(nativeCommandEncoder,
                nativeTextureView.GetNativeHandle());
            return new NativeRenderPass(nativeRenderPass);
        }

        public NativeCommandBuffer Finish()
        {
            if (!isVaild)
            {
                throw new Exception("");
            }
            isVaild = false;
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            NativeCommandBufferType nativeCommandBuffer = Functions.Value.nativeCommandEncoderFinish(nativeCommandEncoder);
            return new NativeCommandBuffer(nativeCommandBuffer);
        }
    }
}
