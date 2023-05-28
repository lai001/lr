using Native;
using Script;
using System.Runtime.InteropServices;

namespace ExampleApplication
{
    [StructLayout(LayoutKind.Sequential)]
    public unsafe class Application
    {
        private NativeShaderModule nativeShaderModule;
        private NativePipelineLayout nativePipelineLayout;
        private NativeRenderPipeline nativeRenderPipeline;

        public UserSscript userSscript;

        public void Initialize()
        {
            if (userSscript != null)
            {
                userSscript.Initialize();
            }
            //nativeShaderModule = NativeDevice.Instance.NativeCreateShaderModule("shader.wgsl", "../../src/shader/triangle.wgsl");
            //nativePipelineLayout = NativeDevice.Instance.NativeCreatePipelineLayout("PipelineLayout");
            //nativeRenderPipeline = NativeDevice.Instance.NativeCreateRenderPipeline("RenderPipeline", nativePipelineLayout, nativeShaderModule, NativeTextureFormat.Bgra8unormSrgb);
        }

        public void RedrawRequested(NativeTextureView nativeTextureView, NativeQueue nativeQueue)
        {
            if (userSscript != null)
            {
                userSscript.RedrawRequested(nativeTextureView, nativeQueue);
            }
            //System.Diagnostics.Debugger.Launch();

            //NativeCommandEncoder nativeCommandEncoder = NativeDevice.Instance.NativeCreateCommandEncoder();

            //using (NativeRenderPass nativeRenderPass = nativeCommandEncoder.BeginRenderPass(nativeTextureView))
            //{
            //    nativeRenderPass.SetPipeline(nativeRenderPipeline);
            //    nativeRenderPass.Draw(new Range<uint>(0, 3), new Range<uint>(0, 1));
            //}

            //NativeCommandBuffer nativeCommandBuffer = nativeCommandEncoder.Finish();
            //nativeCommandBuffer.Submit(nativeQueue);
        }

        public void KeyboardInput(NativeKeyboardInput keyboardInput)
        {
            // Console.WriteLine($"virtualKeyCode:  {keyboardInput.virtualKeyCode}, elementState:  {keyboardInput.elementState}");
            if (userSscript != null)
            {
                userSscript.KeyboardInput(keyboardInput);
            }
        }

        public void CursorMoved(PhysicalPosition physicalPosition)
        {
            //  Console.WriteLine($"CursorMoved:  {physicalPosition.x}, {physicalPosition.y}");
            if (userSscript != null)
            {
                userSscript.CursorMoved(physicalPosition);
            }
        }
    }
}
