using Native;
using Script;
using System.Runtime.InteropServices;
using System;

namespace ExampleApplication
{
    [StructLayout(LayoutKind.Sequential)]
    public unsafe class Application
    {
        private NativeShaderModule nativeShaderModule;
        private NativePipelineLayout nativePipelineLayout;
        private NativeRenderPipeline nativeRenderPipeline;

        public UserSscriptPayload userSscript;

        public void Initialize()
        {
            if (userSscript != null)
            {
                userSscript.Initialize();
            }
        }

        public void Tick(NativeEngine engine)
        {
            if (userSscript != null)
            {
                userSscript.Tick(engine);
            }
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
