using LibreHardwareMonitor.Hardware;

namespace LibreHardwareMonitorWrapper.Lhm;

public class Sensor : BaseHardware
{
    private readonly ISensor _mSensor;

    public Sensor(string id, string name, string info, ISensor sensor, int index) : base(id, name,
        info, index, HardwareType.Sensor)
    {
        _mSensor = sensor;
    }

    public double Value()
    {
        return _mSensor.Value.HasValue ? (double)_mSensor.Value : 0.0;
    }
}