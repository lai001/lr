using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Foundation;
using System.Runtime.InteropServices;

namespace Native
{
    using NativeEngineType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeEngineFunctions
    {
        public delegate* unmanaged<NativeEngineType, int, void> rs_engine_Engine_set_view_mode;
    }

    public unsafe class NativeEngine
    {
        public static NativeEngineFunctions? Functions;

        public NativeEngineType nativeEngine = NativeEngineType.Zero;

        public NativeEngine(NativeEngineType nativeEngine)
        {
            this.nativeEngine = nativeEngine;
        }

        public void SetViewMode(int mode)
        {
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            System.Diagnostics.Debug.Assert(nativeEngine != NativeEngineType.Zero);
            Functions.Value.rs_engine_Engine_set_view_mode(nativeEngine, mode);
        }
    }
}
