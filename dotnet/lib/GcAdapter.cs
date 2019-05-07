using System;
using System.Runtime.InteropServices;

namespace GcAdapterLib
{
    public unsafe class FFI {
        // Unmanaged functions
        [DllImport("libgcadapter_driver.so")]
        public static extern void* gc_context_create(
            Cdecl_OnControllerPlug plugCallback,
            Cdecl_OnControllerUnplug unplugCallback,
            Cdecl_OnControllerState stateCallback
        );

        [DllImport("libgcadapter_driver.so")]
        public static extern void gc_context_delete(void* context);

        // Unmanaged function pointer types
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate IntPtr Cdecl_OnControllerPlug(int adapterId, int port);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Cdecl_OnControllerUnplug(IntPtr ptr);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Cdecl_OnControllerState(IntPtr ptr);
    }

    public class GcAdapter : IDisposable
    {
        // Our managed representation of a controller
        public class ManagedController {
            public int adapterId;
            public int port;

            public int virtualId;

            public int stickX;
            public int stickY;

            public int cStickX;
            public int cStickY;

            public int lAnalog;
            public int rAnalog;

            public int buttons;
        }

        // Finalizer nonsense for FFI
        private bool disposed = false;
        private unsafe void* context;

        // FFI callback handles to prevent GC
        private FFI.Cdecl_OnControllerPlug onControllerPlugHandleFFI;
        private FFI.Cdecl_OnControllerUnplug onControllerUnplugHandleFFI;
        private FFI.Cdecl_OnControllerState onControllerStateHandleFFI;

        public GcAdapter(
            Action<int> onControllerPlug,
            Action<int> onControllerUnplug,
            Action<int> onControllerState
        )
        {
            unsafe
            {
                // Assign FFI callbacks to handles to prevent GC
                onControllerPlugHandleFFI =
                    (adapterId, port) => {
                        // Alloc a controller pinned in memory
                        var controller = new ManagedController();
                        GCHandle handle = GCHandle.Alloc(controller);

                        controller.adapterId = adapterId;
                        controller.port = port;

                        // Just allocate some virtual ID using the adapter
                        // ID and port number. The adapter ID the 8-bit USB
                        // address of the device, so this will never overflow.
                        controller.virtualId = adapterId * 4 + port;

                        return GCHandle.ToIntPtr(handle);
                    };

                onControllerUnplugHandleFFI = 
                    (ptr) => {
                        var handle = GCHandle.FromIntPtr(ptr);
                        var managedController = (ManagedController)handle.Target;

                        Console.WriteLine(managedController.virtualId);
                    };

                onControllerStateHandleFFI =
                    (ptr) => {
                        var handle = GCHandle.FromIntPtr(ptr);
                        var managedController = (ManagedController)handle.Target;

                        Console.WriteLine(managedController.virtualId);
                    };

                // Create the native context
                context = FFI.gc_context_create(
                    onControllerPlugHandleFFI,
                    onControllerUnplugHandleFFI,
                    onControllerStateHandleFFI
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
