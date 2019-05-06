using System;
using System.Runtime.InteropServices;

namespace GcAdapterLib
{
    public unsafe class FFI {
        // Unmanaged functions
        [DllImport("libgcadapter_driver.so")]
        public static extern void* gc_context_create();

        [DllImport("libgcadapter_driver.so")]
        public static extern void gc_context_delete(void* context);

        [DllImport("libgcadapter_driver.so")]
        public static extern void gc_context_set_controller_callbacks(
            void* context,
            Cdecl_OnControllerPlug plugCallback,
            Cdecl_OnControllerUnplug unplugCallback,
            Cdecl_OnControllerState stateCallback
        );

        // Unmanaged function pointer types
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Cdecl_OnControllerPlug(int controllerId);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Cdecl_OnControllerUnplug(int controllerId);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Cdecl_OnControllerState(int controllerId);
    }

    public class GcAdapter : IDisposable
    {
        // FFI finalizer nonsense
        private bool disposed = false;
        private unsafe void* context;

        private FFI.Cdecl_OnControllerPlug onControllerPlugHandle;
        private FFI.Cdecl_OnControllerUnplug onControllerUnplugHandle;
        private FFI.Cdecl_OnControllerState onControllerStateHandle;


        public GcAdapter(
            FFI.Cdecl_OnControllerPlug onControllerPlug,
            FFI.Cdecl_OnControllerUnplug onControllerUnplug,
            FFI.Cdecl_OnControllerState onControllerState
        )
        {
            unsafe
            {
                context = FFI.gc_context_create();

                onControllerPlugHandle = onControllerPlug;
                onControllerUnplugHandle = onControllerUnplug;
                onControllerStateHandle = onControllerState;

                FFI.gc_context_set_controller_callbacks(
                    context,

                    onControllerPlugHandle,
                    onControllerUnplugHandle,
                    onControllerStateHandle
                );
            }
        }

        ~GcAdapter()
        {
            Dispose(true);
        }

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (disposed)
                return;

            if (disposing)
            {
                unsafe
                {
                    FFI.gc_context_delete(context);
                }
            }

            disposed = true;
        }
    }
}
