using Foundation;
using System;
using System.Runtime.InteropServices;
using System.Text;

namespace Native
{
    using NativeShaderModuleType = IntPtr;
    using NativeDeviceType = IntPtr;
    using NativePipelineLayoutType = IntPtr;
    using NativeRenderPipelineType = IntPtr;
    using NativeCommandEncoderType = IntPtr;
    using NativeStringType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeDeviceFunctions
    {
        public delegate* unmanaged<NativeDeviceType, NativeStringType, NativeStringType, NativeShaderModuleType> nativeDeviceCreateShaderModule;
        public delegate* unmanaged<NativeDeviceType, NativeStringType, NativePipelineLayoutType> nativeDeviceCreatePipelineLayout;
        public delegate* unmanaged<NativeDeviceType, NativeStringType, NativePipelineLayoutType, NativeShaderModuleType, NativeTextureFormat, NativeRenderPipelineType> nativeDeviceCreateRenderPipeline;
        public delegate* unmanaged<NativeDeviceType, NativeCommandEncoderType> nativeDeviceCreateCommandEncoder;
    }

    public unsafe class NativeDevice
    {
        public static NativeDeviceFunctions? Functions;

        public static NativeDeviceType nativeDevice = NativeDeviceType.Zero;

        private static NativeDevice? _NativeGpuDevice;

        public NativeDevice()
        {
        }

        public static NativeDevice Instance
        {
            get
            {
                if (_NativeGpuDevice == null)
                {
                    _NativeGpuDevice = new NativeDevice();
                }
                return _NativeGpuDevice;
            }
        }

        public static NativeDeviceType GetNativeHandle()
        {
            return nativeDevice;
        }

        public NativeShaderModule NativeCreateShaderModule(string label, string path)
        {
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            using CString pathCstr = new CString(path);
            using CString labelCstr = new CString(label);
            NativeShaderModuleType nativeShaderModule = Functions.Value.nativeDeviceCreateShaderModule(nativeDevice, labelCstr.GetNativeHandle(), pathCstr.GetNativeHandle());
            return new NativeShaderModule(nativeShaderModule);
        }

        public NativePipelineLayout NativeCreatePipelineLayout(string label)
        {
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            using CString cString = new CString(label);
            NativePipelineLayoutType nativePipelineLayout = Functions.Value.nativeDeviceCreatePipelineLayout(nativeDevice, cString.GetNativeHandle());
            return new NativePipelineLayout(nativePipelineLayout);
        }

        public NativeRenderPipeline NativeCreateRenderPipeline(string label, NativePipelineLayout nativePipelineLayout, NativeShaderModule nativeShaderModule, NativeTextureFormat nativeTextureFormat)
        {
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            using CString cString = new CString(label);
            NativeRenderPipelineType nativeRenderPipeline = Functions.Value.nativeDeviceCreateRenderPipeline(nativeDevice,
                cString.GetNativeHandle(),
                nativePipelineLayout.GetNativeHandle(),
                nativeShaderModule.GetNativeHandle(),
                nativeTextureFormat);
            return new NativeRenderPipeline(nativeRenderPipeline);
        }

        public NativeCommandEncoder NativeCreateCommandEncoder()
        {
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            NativeCommandEncoderType nativeCommandEncoder = Functions.Value.nativeDeviceCreateCommandEncoder(nativeDevice);
            return new NativeCommandEncoder(nativeCommandEncoder);
        }
    }

}
