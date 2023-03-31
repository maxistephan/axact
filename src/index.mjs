import { h, render } from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

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
    let mem_used_mb = (mem_used / 1_000_000).toFixed(2);
    let mem_total_mb = (mem_total / 1_000_000).toFixed(2);
    let mem_free_mb = (mem_total_mb - mem_used_mb).toFixed(2);
    let mem_percent = (mem_used / mem_total * 100).toFixed(2);
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

if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
    console.log("INFO: Using light mode, since system theme prefers light.");
    document.body.classList.add("light-mode");
} else {
    console.log("INFO: Using dark mode, since system theme prefers dark.");
}

let url = new URL("/realtime/ressources", window.location.href);
// http => ws
// https => wss
url.protocol = url.protocol.replace("http", "ws");

let ws = new WebSocket(url.href);
ws.onmessage = (ev) => {
    let json = JSON.parse(ev.data);
    let cpus = json.cpus.Vec32;
    let mem_used = json.mem_used.U64;
    let mem_total = json.mem_total.U64;
    render(html`<${CpuStats} cpus=${cpus}></${CpuStats}>`, document.getElementById("cpu_stats"));
    render(html`<${MemStats} mem_used=${mem_used} mem_total=${mem_total}></${MemStats}>`, document.getElementById("mem_stats"));
}
