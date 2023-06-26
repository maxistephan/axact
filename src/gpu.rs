use nvml_wrapper::{
    enum_wrappers::device::TemperatureSensor,
    error::NvmlError, Device
};
use nvml_wrapper::NVML;

pub fn get_gpu_avg_temp()  -> Result<f32, NvmlError> {
    let temps: Vec<f32> = get_gpu_temps().unwrap();
    let cnt: f32 = temps.len() as f32;
    let mut nums: f32 = 0f32;
    for element in temps.iter() {
        nums += *element;
    }
    Ok(nums / cnt)
}

fn get_gpu_temps() -> Result<Vec<f32>, NvmlError> {
    let nvml: NVML = NVML::init()?;
    let device_count: u32 = nvml.device_count()?;
    let mut temperatures: Vec<f32> = vec![];

    for i in 0..device_count {
        let device: Device = nvml.device_by_index(i)?;
        let temp: u32 = read_gpu_temp(&device)?;
        temperatures.push(temp as f32);
    }
    Ok(temperatures)
}

fn read_gpu_temp(device: &Device) -> Result<u32, NvmlError> {
    device.temperature(TemperatureSensor::Gpu)
}
