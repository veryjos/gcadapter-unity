using System;
using System.Collections.Concurrent;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
struct GcControllerState
{
    public bool pluggedIn;
    public uint buttons;
    public float x;
    public float y;
    public float cx;
    public float cy;
    public float l;
    public float r;
};

public class GamecubeAdapter
{
    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    delegate void OnControllerPluggedCallback(ulong controllerId);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    delegate void OnControllerUnpluggedCallback(ulong controllerId);

    [DllImport("GcAdapterLibUSBDriver")]
    private static extern UIntPtr gc_create_context(
        [MarshalAs(UnmanagedType.FunctionPtr)]
        OnControllerPluggedCallback adapterPluggedIn,

        [MarshalAs(UnmanagedType.FunctionPtr)]
        OnControllerUnpluggedCallback adapterUnplugged
    );

    [DllImport("GcAdapterLibUSBDriver")]
    private static extern void gc_destroy_context(UIntPtr context);

    // returns the latest state for the given context and controller id
    //
    // controller states are polled at 100hz (usb limit)
    [DllImport("GcAdapterLibUSBDriver")]
    private static extern GcControllerState gc_get_latest_controller_state(UIntPtr context, ulong controllerId);

    private UIntPtr context;
    private ConcurrentDictionary<ulong, GcControllerState> controllerState =
        new ConcurrentDictionary<ulong, GcControllerState>();

    public void Start()
    {
        context = gc_create_context(OnControllerPlugged, OnControllerUnplugged);
    }

    public void Stop()
    {
        gc_destroy_context(context);
    }

    public void GetControllerStates()
    {
        foreach (var id in controllerState.Keys)
        {
            controllerState[id] = gc_get_latest_controller_state(context, id);
        }
    }

    // callback will be invoked by a different thread
    private void OnControllerPlugged(ulong controllerId)
    {
        controllerState.TryAdd(controllerId, new GcControllerState());
    }

    // callback will be invoked by a different thread
    private void OnControllerUnplugged(ulong controllerId)
    {
        GcControllerState val;
        controllerState.TryRemove(controllerId, out val);
    }
}