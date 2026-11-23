// Win32 helpers for the W-1 overlay matrix. Loaded by
// scripts/overlay-matrix-windows.ps1 with Add-Type. Dev tooling only.
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public static class LlDisplay
{
    public struct Monitor
    {
        public string Device;
        public string Description;
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
        public uint Dpi;
        public bool Primary;
        public bool IsCurrentDesktop;
    }

    public struct Source
    {
        public string Device;
        public LUID AdapterId;
        public uint Id;
        public int ScalePercent;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LUID { public uint LowPart; public int HighPart; }

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }

    [StructLayout(LayoutKind.Sequential)]
    public struct DISPLAYCONFIG_DEVICE_INFO_HEADER
    {
        public int type;
        public uint size;
        public LUID adapterId;
        public uint id;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_RATIONAL { public uint Numerator; public uint Denominator; }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_PATH_SOURCE_INFO
    {
        public LUID adapterId;
        public uint id;
        public uint modeInfoIdx;
        public uint statusFlags;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_PATH_TARGET_INFO
    {
        public LUID adapterId;
        public uint id;
        public uint modeInfoIdx;
        public uint outputTechnology;
        public uint rotation;
        public uint scaling;
        public DISPLAYCONFIG_RATIONAL refreshRate;
        public uint scanLineOrdering;
        public int targetAvailable;
        public uint statusFlags;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DISPLAYCONFIG_PATH_INFO
    {
        public DISPLAYCONFIG_PATH_SOURCE_INFO sourceInfo;
        public DISPLAYCONFIG_PATH_TARGET_INFO targetInfo;
        public uint flags;
    }

    [StructLayout(LayoutKind.Explicit, Size = 64)]
    struct DISPLAYCONFIG_MODE_INFO { }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct DISPLAYCONFIG_SOURCE_DEVICE_NAME
    {
        public DISPLAYCONFIG_DEVICE_INFO_HEADER header;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string viewGdiDeviceName;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DPI_SCALE_INFO
    {
        public DISPLAYCONFIG_DEVICE_INFO_HEADER header;
        public int minScaleRel;
        public int curScaleRel;
        public int maxScaleRel;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DPI_SCALE_SET
    {
        public DISPLAYCONFIG_DEVICE_INFO_HEADER header;
        public int scaleRel;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct MONITORINFOEX
    {
        public int cbSize;
        public RECT rcMonitor;
        public RECT rcWork;
        public uint dwFlags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string szDevice;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct DISPLAY_DEVICE
    {
        public int cb;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string DeviceName;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
        public string DeviceString;
        public uint StateFlags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
        public string DeviceID;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
        public string DeviceKey;
    }

    delegate bool MonitorEnumProc(IntPtr hMonitor, IntPtr hdc, IntPtr rect, IntPtr data);

    [DllImport("user32.dll")]
    static extern bool EnumDisplayMonitors(IntPtr hdc, IntPtr clip, MonitorEnumProc proc, IntPtr data);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern bool GetMonitorInfoW(IntPtr monitor, ref MONITORINFOEX info);
    [DllImport("shcore.dll")]
    static extern int GetDpiForMonitor(IntPtr monitor, int dpiType, out uint dpiX, out uint dpiY);
    [DllImport("shcore.dll")]
    static extern int SetProcessDpiAwareness(int value);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern bool EnumDisplayDevicesW(string device, uint devNum, ref DISPLAY_DEVICE info, uint flags);
    [DllImport("user32.dll")]
    static extern int GetDisplayConfigBufferSizes(uint flags, out uint pathCount, out uint modeCount);
    [DllImport("user32.dll")]
    static extern int QueryDisplayConfig(uint flags, ref uint pathCount, [Out] DISPLAYCONFIG_PATH_INFO[] paths,
        ref uint modeCount, IntPtr modes, IntPtr topologyId);
    [DllImport("user32.dll")]
    static extern int DisplayConfigGetDeviceInfo(ref DISPLAYCONFIG_DEVICE_INFO_HEADER packet);
    [DllImport("user32.dll")]
    static extern int DisplayConfigGetDeviceInfo(ref DISPLAYCONFIG_SOURCE_DEVICE_NAME packet);
    [DllImport("user32.dll")]
    static extern int DisplayConfigGetDeviceInfo(ref DPI_SCALE_INFO packet);
    [DllImport("user32.dll")]
    static extern int DisplayConfigSetDeviceInfo(ref DPI_SCALE_SET packet);

    const uint QDC_ONLY_ACTIVE_PATHS = 0x00000002;
    static readonly int[] DpiVals = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };

    public static List<Monitor> Monitors()
    {
        SetProcessDpiAwareness(2);
        var list = new List<Monitor>();
        EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, delegate(IntPtr hMonitor, IntPtr hdc, IntPtr rect, IntPtr data)
        {
            var info = new MONITORINFOEX();
            info.cbSize = Marshal.SizeOf(typeof(MONITORINFOEX));
            if (!GetMonitorInfoW(hMonitor, ref info)) return true;
            uint dpiX, dpiY;
            if (GetDpiForMonitor(hMonitor, 0, out dpiX, out dpiY) != 0) dpiX = 96;
            string description = "";
            var dev = new DISPLAY_DEVICE();
            dev.cb = Marshal.SizeOf(typeof(DISPLAY_DEVICE));
            if (EnumDisplayDevicesW(info.szDevice, 0, ref dev, 0)) description = dev.DeviceString;
            list.Add(new Monitor
            {
                Device = info.szDevice,
                Description = description,
                Left = info.rcMonitor.Left,
                Top = info.rcMonitor.Top,
                Right = info.rcMonitor.Right,
                Bottom = info.rcMonitor.Bottom,
                Dpi = dpiX,
                Primary = (info.dwFlags & 1) != 0,
            });
            return true;
        }, IntPtr.Zero);
        return list;
    }

    public static List<Source> Sources()
    {
        var result = new List<Source>();
        uint paths = 0, modes = 0;
        if (GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, out paths, out modes) != 0) return result;
        var pathArray = new DISPLAYCONFIG_PATH_INFO[paths];
        var modeSize = Marshal.SizeOf(typeof(DISPLAYCONFIG_MODE_INFO));
        var modePtr = Marshal.AllocHGlobal((int)modes * modeSize);
        try
        {
            if (QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS, ref paths, pathArray, ref modes, modePtr, IntPtr.Zero) != 0)
                return result;
            for (uint i = 0; i < paths; i++)
            {
                var path = pathArray[i];
                var name = new DISPLAYCONFIG_SOURCE_DEVICE_NAME();
                name.header.type = 1;
                name.header.size = (uint)Marshal.SizeOf(typeof(DISPLAYCONFIG_SOURCE_DEVICE_NAME));
                name.header.adapterId = path.sourceInfo.adapterId;
                name.header.id = path.sourceInfo.id;
                if (DisplayConfigGetDeviceInfo(ref name) != 0) continue;
                var scale = GetScalePercent(path.sourceInfo.adapterId, path.sourceInfo.id);
                result.Add(new Source
                {
                    Device = name.viewGdiDeviceName,
                    AdapterId = path.sourceInfo.adapterId,
                    Id = path.sourceInfo.id,
                    ScalePercent = scale,
                });
            }
        }
        finally
        {
            Marshal.FreeHGlobal(modePtr);
        }
        return result;
    }

    public static int GetScalePercent(LUID adapter, uint id)
    {
        var packet = new DPI_SCALE_INFO();
        packet.header.type = -3;
        packet.header.size = (uint)Marshal.SizeOf(typeof(DPI_SCALE_INFO));
        packet.header.adapterId = adapter;
        packet.header.id = id;
        if (DisplayConfigGetDeviceInfo(ref packet) != 0) return 0;
        var minAbs = Math.Abs(packet.minScaleRel);
        var index = minAbs + packet.curScaleRel;
        if (index < 0 || index >= DpiVals.Length) return 0;
        return DpiVals[index];
    }

    public static bool SetScalePercent(string device, int percent)
    {
        foreach (var source in Sources())
        {
            if (!string.Equals(source.Device, device, StringComparison.OrdinalIgnoreCase)) continue;
            if (source.ScalePercent == percent) return true;
            var packet = new DPI_SCALE_INFO();
            packet.header.type = -3;
            packet.header.size = (uint)Marshal.SizeOf(typeof(DPI_SCALE_INFO));
            packet.header.adapterId = source.AdapterId;
            packet.header.id = source.Id;
            if (DisplayConfigGetDeviceInfo(ref packet) != 0) return false;
            var minAbs = Math.Abs(packet.minScaleRel);
            var recommended = DpiVals[Math.Min(minAbs, DpiVals.Length - 1)];
            var target = Array.IndexOf(DpiVals, percent);
            if (target < 0) return false;
            var recommendedIndex = Array.IndexOf(DpiVals, recommended);
            if (recommendedIndex < 0) return false;
            var set = new DPI_SCALE_SET();
            set.header.type = -4;
            set.header.size = (uint)Marshal.SizeOf(typeof(DPI_SCALE_SET));
            set.header.adapterId = source.AdapterId;
            set.header.id = source.Id;
            set.scaleRel = target - recommendedIndex;
            return DisplayConfigSetDeviceInfo(ref set) == 0;
        }
        return false;
    }
}

public static class LlWindow
{
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern IntPtr FindWindowW(string cls, string title);
    [DllImport("user32.dll")]
    static extern long GetWindowLongPtrW(IntPtr hwnd, int index);
    [DllImport("user32.dll")]
    static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")]
    static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")]
    static extern IntPtr WindowFromPoint(POINT point);
    [DllImport("user32.dll")]
    static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")]
    static extern bool GetCursorPos(out POINT point);
    [DllImport("user32.dll")]
    static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern bool GetWindowTextW(IntPtr hwnd, System.Text.StringBuilder text, int max);
    [DllImport("user32.dll")]
    static extern bool IsWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
    [DllImport("user32.dll")]
    static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")]
    static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    [DllImport("user32.dll")]
    static extern bool AttachThreadInput(uint attach, uint attachTo, bool doAttach);
    [DllImport("user32.dll")]
    static extern bool BringWindowToTop(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
    [DllImport("kernel32.dll")]
    static extern uint GetCurrentThreadId();

    public const long WS_EX_TOPMOST = 0x00000008;
    public const long WS_EX_NOACTIVATE = 0x08000000;
    public const long WS_EX_TOOLWINDOW = 0x00000080;

    public static long ExStyle(IntPtr hwnd) { return GetWindowLongPtrW(hwnd, -20); }
    public static bool TryRect(IntPtr hwnd, out RECT rect) { return GetWindowRect(hwnd, out rect); }
    public static IntPtr Foreground() { return GetForegroundWindow(); }
    public static IntPtr At(int x, int y) { var p = new POINT { X = x, Y = y }; return WindowFromPoint(p); }
    public static bool Click(int x, int y)
    {
        POINT old;
        GetCursorPos(out old);
        var moved = SetCursorPos(x, y);
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
        SetCursorPos(old.X, old.Y);
        return moved;
    }

    /// Press at one point, move, release. Used to drag the overlay grip through
    /// the real OS move loop.
    public static bool Drag(int x1, int y1, int x2, int y2)
    {
        POINT old;
        GetCursorPos(out old);
        var moved = SetCursorPos(x1, y1);
        System.Threading.Thread.Sleep(150);
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
        System.Threading.Thread.Sleep(200);
        SetCursorPos(x2, y2);
        System.Threading.Thread.Sleep(250);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
        System.Threading.Thread.Sleep(250);
        SetCursorPos(old.X, old.Y);
        return moved;
    }
    public static string Title(IntPtr hwnd)
    {
        var text = new System.Text.StringBuilder(512);
        GetWindowTextW(hwnd, text, text.Capacity);
        return text.ToString();
    }
    public static bool Alive(IntPtr hwnd) { return IsWindow(hwnd) && IsWindowVisible(hwnd); }
    public static bool Capture(IntPtr hwnd, IntPtr hdc) { return PrintWindow(hwnd, hdc, 2); }
    public static bool ForceForeground(IntPtr hwnd)
    {
        keybd_event(0x12, 0, 0, UIntPtr.Zero);
        keybd_event(0x12, 0, 2, UIntPtr.Zero);
        var foreground = GetForegroundWindow();
        uint foregroundPid;
        var foregroundThread = GetWindowThreadProcessId(foreground, out foregroundPid);
        var currentThread = GetCurrentThreadId();
        var attached = AttachThreadInput(currentThread, foregroundThread, true);
        try
        {
            BringWindowToTop(hwnd);
            SetForegroundWindow(hwnd);
        }
        finally
        {
            if (attached) AttachThreadInput(currentThread, foregroundThread, false);
        }
        return GetForegroundWindow() == hwnd;
    }
    public static bool Move(IntPtr hwnd, int x, int y, int cx, int cy, bool topmost)
    {
        return SetWindowPos(hwnd, topmost ? new IntPtr(-1) : new IntPtr(-2), x, y, cx, cy, 0x0010);
    }
    public static bool OnCurrentVirtualDesktop(IntPtr hwnd)
    {
        try
        {
            var manager = (IVirtualDesktopManager)new VirtualDesktopManager();
            bool onCurrent;
            return manager.IsWindowOnCurrentVirtualDesktop(hwnd, out onCurrent) == 0 && onCurrent;
        }
        catch
        {
            return false;
        }
    }

    [ComImport, Guid("a5cd92ff-29be-454c-8d04-d82879fb3f1b"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IVirtualDesktopManager
    {
        [PreserveSig] int IsWindowOnCurrentVirtualDesktop(IntPtr topLevelWindow, out bool onCurrent);
        [PreserveSig] int GetWindowDesktopId(IntPtr topLevelWindow, out Guid desktopId);
        [PreserveSig] int MoveWindowToDesktop(IntPtr topLevelWindow, ref Guid desktopId);
    }

    [ComImport, Guid("aa509086-5ca9-4c25-8f95-589d3c07b48a")]
    class VirtualDesktopManager { }
}

// A borderless, non-topmost full-screen window used as the "video/call app"
// foreground proxy. It is not a second UI framework: one plain Win32 window.
public static class LlProxyWindow
{
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct WNDCLASSEX
    {
        public uint cbSize;
        public uint style;
        public IntPtr lpfnWndProc;
        public int cbClsExtra;
        public int cbWndExtra;
        public IntPtr hInstance;
        public IntPtr hIcon;
        public IntPtr hCursor;
        public IntPtr hbrBackground;
        [MarshalAs(UnmanagedType.LPWStr)] public string lpszMenuName;
        [MarshalAs(UnmanagedType.LPWStr)] public string lpszClassName;
        public IntPtr hIconSm;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct MSG
    {
        public IntPtr hwnd;
        public uint message;
        public IntPtr wParam;
        public IntPtr lParam;
        public uint time;
        public int x;
        public int y;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    delegate IntPtr WndProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern ushort RegisterClassExW(ref WNDCLASSEX info);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern IntPtr CreateWindowExW(uint exStyle, string className, string windowName, uint style,
        int x, int y, int width, int height, IntPtr parent, IntPtr menu, IntPtr instance, IntPtr param);
    [DllImport("user32.dll")]
    static extern bool DestroyWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern IntPtr DefWindowProcW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")]
    static extern bool ShowWindow(IntPtr hwnd, int cmd);
    [DllImport("user32.dll")]
    static extern bool UpdateWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern IntPtr SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern bool PeekMessageW(out MSG msg, IntPtr hwnd, uint min, uint max, uint remove);
    [DllImport("user32.dll")]
    static extern bool TranslateMessage(ref MSG msg);
    [DllImport("user32.dll")]
    static extern IntPtr DispatchMessageW(ref MSG msg);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
    static extern IntPtr GetModuleHandleW(string name);
    [DllImport("gdi32.dll")]
    static extern IntPtr CreateSolidBrush(uint color);

    static readonly WndProc Proc = DefProc;
    const string ClassName = "LiveLinguistVideoProxy";
    static bool registered;
    static IntPtr brush;

    static IntPtr DefProc(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    }

    public static IntPtr Show(int left, int top, int right, int bottom)
    {
        var instance = GetModuleHandleW(null);
        if (!registered)
        {
            if (brush == IntPtr.Zero) brush = CreateSolidBrush(0x00302010);
            var info = new WNDCLASSEX();
            info.cbSize = (uint)Marshal.SizeOf(typeof(WNDCLASSEX));
            info.lpfnWndProc = Marshal.GetFunctionPointerForDelegate(Proc);
            info.hInstance = instance;
            info.hbrBackground = brush;
            info.lpszClassName = ClassName;
            RegisterClassExW(ref info);
            registered = true;
        }
        const uint WS_POPUP = 0x80000000;
        const uint WS_VISIBLE = 0x10000000;
        const uint WS_EX_APPWINDOW = 0x00040000;
        var hwnd = CreateWindowExW(WS_EX_APPWINDOW, ClassName, "Video call (proxy)",
            WS_POPUP | WS_VISIBLE, left, top, right - left, bottom - top,
            IntPtr.Zero, IntPtr.Zero, instance, IntPtr.Zero);
        if (hwnd == IntPtr.Zero) return hwnd;
        ShowWindow(hwnd, 5);
        UpdateWindow(hwnd);
        MSG msg;
        for (var i = 0; i < 60; i++)
        {
            while (PeekMessageW(out msg, IntPtr.Zero, 0, 0, 1))
            {
                TranslateMessage(ref msg);
                DispatchMessageW(ref msg);
            }
            System.Threading.Thread.Sleep(10);
        }
        LlWindow.ForceForeground(hwnd);
        return hwnd;
    }

    public static void Close(IntPtr hwnd)
    {
        if (hwnd != IntPtr.Zero) DestroyWindow(hwnd);
    }
}
