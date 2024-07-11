using Native;
using Script;
using System;

namespace UserScript
{
    public class MyUserScript : IUserScript, IDisposable
    {
        public string Name { get => "MyUserScript"; }
        public string Description { get => "MyUserScript"; }

        public void Initialize()
        {
        }

        public void CursorMoved(PhysicalPosition physicalPosition)
        {
        }

        public void KeyboardInput(NativeKeyboardInput keyboardInput)
        {
        }

        public void Dispose()
        {
        }

        public void Tick(NativeEngine engine)
        {
            engine.SetViewMode(0);
        }
    }
}