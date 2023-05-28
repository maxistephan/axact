use std::process::Command;

const LIQUIDCTL: &str = "/usr/bin/liquidctl";
const DELTA_TEMP: f32 = 5f32;
const DELTA_SPEED: i32 = 4;
const TARGET_CORE_TEMP: f32 = 55f32;
const TARGET_GPU_TEMP: u32 = 65;
const MIN_FAN_PERCENTAGE: i32 = 20;
const MAX_FAN_PERCENTAGE: i32 = 70;

pub fn liquidctl_modify_fan_speed(cpu_temp: f32, gpu_temp: u32, last_speed: i32) -> i32 {
    let mut new_speed: i32 = last_speed;

    // get the biggest delta
    let gpu_delta: f32 = (gpu_temp / TARGET_GPU_TEMP) as f32;
    let cpu_delta: f32 = cpu_temp / TARGET_CORE_TEMP;

    let mut source_temp: f32 = cpu_temp;
    let mut target_temp: f32 = TARGET_CORE_TEMP;

    if cpu_delta < gpu_delta {
        source_temp = gpu_temp as f32;
        target_temp = TARGET_GPU_TEMP as f32;
    }

    // Compute new fan speed based on CPU temp
    if source_temp > target_temp + DELTA_TEMP {
        new_speed = last_speed + DELTA_SPEED;
    } else if source_temp <= target_temp {
        new_speed = last_speed - DELTA_SPEED;
    }

    // Check fan speed for validity
    if new_speed <= MIN_FAN_PERCENTAGE {
        return MIN_FAN_PERCENTAGE;
    } else if new_speed >= MAX_FAN_PERCENTAGE {
        return MAX_FAN_PERCENTAGE;
    } else if new_speed == last_speed {
        return new_speed;
    }

    // Run liquidctl
    println!("Adjusting fan speed from {} to {} ...", last_speed, new_speed);
    run_liquidctl(new_speed);

    // Return new fan speed
    new_speed
}

pub fn run_liquidctl(fan_speed: i32) {
    println!("Running liquidctl with new fan speed of {}% ...", fan_speed);
    Command::new(LIQUIDCTL)
        .arg("set")
        .arg("--match")
        .arg("Smart Device")
        .arg("sync")
        .arg("speed")
        .arg(fan_speed.to_string())
        .spawn()
        .expect("Failed to start liquidctl.");
}

pub fn initialize_liquidctl() {
    println!("Initializing {} ...", LIQUIDCTL);
    Command::new(LIQUIDCTL)
        .arg("initialize")
        .arg("all")
        .spawn()
        .expect("Failed to start liquidctl.");
}
