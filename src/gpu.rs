use nvml_wrapper::{
    enum_wrappers::device::TemperatureSensor,
    error::NvmlError, Device
};
use nvml_wrapper::NVML;

pub fn get_gpu_avg_temp()  -> Result<u32, NvmlError> {
    let temps: Vec<u32> = get_gpu_temps().unwrap();
    let cnt: u32 = temps.len() as u32;
    let mut nums: u32 = 0;
    for element in temps.iter() {
        nums += *element;
    }
    Ok((nums / cnt) as u32)
}

fn get_gpu_temps() -> Result<Vec<u32>, NvmlError> {
    let nvml: NVML = NVML::init().unwrap();
    let device_count: u32 = nvml.device_count().unwrap();
    let mut temperatures: Vec<u32> = vec![];

    for i in 0..device_count {
        let device: Device = nvml.device_by_index(i).unwrap();
        let temp: u32 = read_gpu_temp(&device).unwrap();
        temperatures.push(temp);
    }
    Ok(temperatures)

}

fn read_gpu_temp(device: &Device) -> Result<u32, NvmlError> {
    device.temperature(TemperatureSensor::Gpu)
}
