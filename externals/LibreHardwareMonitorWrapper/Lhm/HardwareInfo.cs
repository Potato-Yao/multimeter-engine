using LibreHardwareMonitor.Hardware;

namespace LibreHardwareMonitorWrapper.Lhm;

/// <summary>
/// Represents hardware information such as CPU name, GPU name, etc.
/// Contains string-based properties from the hardware.
/// </summary>
public class HardwareInfo : BaseHardware
{
    private readonly IHardware _mHardware;

    public HardwareInfo(string id, string name, string info, IHardware hardware, int index)
        : base(id, name, info, index, HardwareType.Info)
    {
        _mHardware = hardware;
        HardwareTypeName = hardware.HardwareType.ToString();
    }

    /// <summary>
    /// The type of hardware (CPU, GPU, Motherboard, etc.)
    /// </summary>
    public string HardwareTypeName { get; }

    /// <summary>
    /// Returns the name of the hardware (e.g., "AMD Ryzen 9 5900X", "NVIDIA GeForce RTX 3080")
    /// </summary>
    public string GetName()
    {
        return _mHardware.Name;
    }

    /// <summary>
    /// Returns the identifier path of the hardware
    /// </summary>
    public string GetIdentifier()
    {
        return _mHardware.Identifier.ToString();
    }
}
