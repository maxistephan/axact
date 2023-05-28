import { h, render } from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

// Light mode
// if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
//     console.log("INFO: Using light mode, since system theme prefers light.");
//     document.body.classList.add("light-mode");
// } else {
//     console.log("INFO: Using dark mode, since system theme prefers dark.");
// }

function percentageToColor(percentage, maxHue = 0, minHue = 120) {
    const hue = 120 - percentage;
    return `hsl(${hue}, 100%, 50%)`;
}

const map = (x, in_min, in_max, out_min, out_max) => {
return ((x - in_min) * (out_max - out_min)) / (in_max - in_min) + out_min;
};

const mapper = (range) => {
    return (x) => {
        return map(x, 0, 100, 0, range);
    };
};

const m = mapper(120);

function Proc({ cpu, total }) {
    return html`<div
      class="bar-inner"
      style="
        height: ${cpu}%;
        opacity: ${cpu / 100};
        background: ${percentageToColor(m(cpu))}
      "
    ></div>`;
}

function CpuStats({ cpus }) {
    if (cpus) {
        return html`
          <div class="procs">
            ${cpus
              .map((cpu, i) => {
                return html`<div class="full-bar">
                    <div class="bar">
                        <${Proc} cpu=${cpu} total=${cpus.length} />
                        <label>${cpu.toFixed(2)}%</label>
                    </div>
                    <label>${i}</label>
                </div>`;
              })}
          </div>
        `;
      }
}

function MemStats({ mem_used, mem_total }) {
    const mem_used_mb = (mem_used / 1_000_000).toFixed(2);
    const mem_total_mb = (mem_total / 1_000_000).toFixed(2);
    const mem_free_mb = (mem_total_mb - mem_used_mb).toFixed(2);
    const mem_percent = (mem_used / mem_total * 100).toFixed(2);
    return html`<div class="full-mem-bar">
        <div class="mem-bar">
            <label>${mem_percent}%</label>
            <div
                class="mem-bar-inner"
                style='
                    width: ${mem_percent}%;
                    opacity: ${mem_percent / 100};
                    background: ${percentageToColor(m(mem_percent))};
                '
            ></div>
        </div>
        <div class="mem-label">
            <label>Total: ${mem_total_mb} MB</label>
            <label>Free: ${mem_free_mb} MB</label>
        </div>
    </div>`;
}

function TempStats({ cpu_temp, gpu_temp, fan_speed }) {
    return html`<div class="full-temp-bar">
        <div class="temp-bar">
            <label>${fan_speed}%</label>
            <div
                class="temp-bar-inner"
                style='
                    width: ${fan_speed}%;
                    opacity: ${fan_speed / 100};
                    background: ${percentageToColor(m(fan_speed))};
                '
            ></div>
        </div>
        <div class="temp-label">
            <label>CPU: ${cpu_temp} °C</label>
            <label>GPU: ${gpu_temp} °C</label>
        </div>
    </div>`;
}

function handle_cpus(cpus_json) {
    const cpus = cpus_json.CPUData;
    render(
        html`<${CpuStats} cpus=${cpus}></${CpuStats}>`,
        document.getElementById("cpu_stats")
    );
}

function handle_mem(mem_json) {
    const mem_used = mem_json.MemData.mem_used;
    const mem_total = mem_json.MemData.mem_total;
    render(
        html`<${MemStats} mem_used=${mem_used} mem_total=${mem_total}></${MemStats}>`,
        document.getElementById("mem_stats")
    );
}

function handle_temp(temp_json) {
    const cpu_temp = temp_json.cpu_temp.Temperature;
    const gpu_temp = temp_json.gpu_temp.Temperature;
    const fan_speed = temp_json.fan_speed.FanSpeed;
    render(
        html`<${TempStats} cpu_temp=${cpu_temp} gpu_temp=${gpu_temp} fan_speed=${fan_speed}></${TempStats}>`,
        document.getElementById("temp_stats")
    );
}

let ressource_url = new URL("/realtime/ressources", window.location.href);
let temp_url = new URL("/realtime/temperature", window.location.href);
// http => ws
// https => wss
ressource_url.protocol = ressource_url.protocol.replace("http", "ws");
temp_url.protocol = temp_url.protocol.replace("http", "ws");

let ressource_ws = new WebSocket(ressource_url.href);
let temp_ws = new WebSocket(temp_url.href);

ressource_ws.onmessage = (ev) => {
    const json = JSON.parse(ev.data);
    handle_cpus(json.Ressource.cpu);
    handle_mem(json.Ressource.mem);
}

temp_ws.onmessage = (ev) => {
    const json = JSON.parse(ev.data);
    handle_temp(json.Temperature);
}
