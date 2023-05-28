using Native;
using Script;
using System;

namespace UserScript
{
    public class MyUserScript : IUserScript, IDisposable
    {
        public string Name { get => "MyUserScript"; }
        public string Description { get => "MyUserScript"; }

        private NativeShaderModule nativeShaderModule;
        private NativePipelineLayout nativePipelineLayout;
        private NativeRenderPipeline nativeRenderPipeline;

        public void Initialize()
        {
            nativeShaderModule = NativeDevice.Instance.NativeCreateShaderModule("shader.wgsl", "../../src/shader/triangle.wgsl");
            nativePipelineLayout = NativeDevice.Instance.NativeCreatePipelineLayout("PipelineLayout");
            nativeRenderPipeline = NativeDevice.Instance.NativeCreateRenderPipeline("RenderPipeline", nativePipelineLayout, nativeShaderModule, NativeTextureFormat.Bgra8unormSrgb);
        }

        public void CursorMoved(PhysicalPosition physicalPosition)
        {

        }

        public void KeyboardInput(NativeKeyboardInput keyboardInput)
        {

        }

        public void RedrawRequested(NativeTextureView nativeTextureView, NativeQueue nativeQueue)
        {
            bool isDraw = true;
            if (isDraw)
            {
                NativeCommandEncoder nativeCommandEncoder = NativeDevice.Instance.NativeCreateCommandEncoder();

                using (NativeRenderPass nativeRenderPass = nativeCommandEncoder.BeginRenderPass(nativeTextureView))
                {
                    nativeRenderPass.SetPipeline(nativeRenderPipeline);
                    nativeRenderPass.Draw(new Range<uint>(0, 3), new Range<uint>(0, 1));
                }

                NativeCommandBuffer nativeCommandBuffer = nativeCommandEncoder.Finish();
                nativeCommandBuffer.Submit(nativeQueue);
            }
        }

        public void Dispose()
        {


        }
    }
}