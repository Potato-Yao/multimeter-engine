using LibreHardwareMonitor.Hardware;

namespace LibreHardwareMonitorWrapper.Lhm;

public class HardwareResearcher : IVisitor
{
    private readonly Computer _mComputer = new()
    {
        IsBatteryEnabled = true,
        IsControllerEnabled = true,
        IsCpuEnabled = true,
        IsGpuEnabled = true,
        IsMemoryEnabled = true,
        IsMotherboardEnabled = true,
        IsNetworkEnabled = true,
        IsPsuEnabled = true,
        IsStorageEnabled = true
    };

    private bool _isStarted;

    /////////////////////////// Visitor ///////////////////////////
    public void VisitComputer(IComputer computer)
    {
        computer.Traverse(this);
    }

    public void VisitHardware(IHardware hardware)
    {
        hardware.Update();
        foreach (var subHardware in hardware.SubHardware)
            subHardware.Accept(this);
    }

    public void VisitSensor(ISensor sensor)
    {
    }

    public void VisitParameter(IParameter parameter)
    {
    }

    public void Start()
    {
        if (_isStarted)
            return;

        _mComputer.Open();
        _mComputer.Accept(this);

        _isStarted = true;
    }

    public void Stop()
    {
        Logger.Info("Shutdown lhm");

        if (!_isStarted)
            return;

        _mComputer.Close();
        _isStarted = false;
    }


    public List<BaseHardware> GetHardwareList()
    {
        if (!_isStarted)
            throw new Exception();

        // ugly, has no reusability, i use array instead
        // var nbControl = 0;
        // var nbFan = 0;
        // var nbTemp = 0;
        // var nbTot = 0;
        var totInd = 0;
        var counterList = new int[Enum.GetNames<SensorType>().Length + 1];
        var hardwareList = new List<BaseHardware>();

        void AddHardware(ISensor sensor)
        {
            Logger.Debug("detect sensor's type " + sensor.SensorType + " with name " + sensor.Name + " with id " +
                         sensor.Identifier);
            // if (sensor.SensorType != SensorType.Control && sensor.SensorType != SensorType.Temperature &&
            //     sensor.SensorType != SensorType.Fan)
            //     return;

            var id = sensor.Identifier.ToString();
            var name = sensor.Name.Length > 0 ? sensor.Name : sensor.SensorType.ToString();

            if (sensor.SensorType == SensorType.Control)
            {
                // todo
            }
            else
            {
                id = sensor.SensorType.ToString() + counterList[(int)sensor.SensorType];
                hardwareList.Add(new Sensor(id, sensor.Name, sensor.SensorType.ToString(), sensor, totInd));

                ++totInd;
                ++counterList[(int)sensor.SensorType];
            }

            // the original implementation is so fucking ugly
            //     switch (sensor.SensorType)
            //     {
            //         case SensorType.Control:
            //             id ??= SensorType.Control.ToString() + nbControl;
            //             hardwareList.Add(new Control(id, name, name, sensor, nbTot));
            //             nbControl += 1;
            //             break;
            //         case SensorType.Fan:
            //             id ??= SensorType.Fan.ToString() + nbFan;
            //             hardwareList.Add(new Sensor(id, name, name, sensor, nbTot, HardwareType.Fan));
            //             nbFan += 1;
            //             break;
            //         case SensorType.Temperature:
            //             id ??= SensorType.Temperature.ToString() + nbTemp;
            //             hardwareList.Add(new Sensor(id, name, name, sensor, nbTot, HardwareType.Temp));
            //             nbTemp += 1;
            //             break;
            //
            //         default: throw new Exception("wrong sensor type");
            //     }
            //
            //     nbTot += 1;
        }

        var hardwareArray = _mComputer.Hardware;
        foreach (var hardware in hardwareArray)
        {
            Logger.Info("reach hardware: " + hardware.Name + " with type " + hardware.HardwareType);
            var sensorArray = hardware.Sensors;
            foreach (var sensor in sensorArray) AddHardware(sensor);

            var subHardwareArray = hardware.SubHardware;
            foreach (var subHardware in subHardwareArray)
            {
                var subSensorArray = subHardware.Sensors;
                foreach (var subSensor in subSensorArray) AddHardware(subSensor);
            }
        }

        // Logger.Info("Control: " + nbControl + ", Fans: " + nbFan + ", Temps: " + nbTemp);
        foreach (var hardware in hardwareList)
        {
            Logger.Info("Restore type " + hardware.Type + " with name " + hardware.Name);
        }

        return hardwareList;
    }

    public void Update()
    {
        _mComputer.Accept(this);
    }
}