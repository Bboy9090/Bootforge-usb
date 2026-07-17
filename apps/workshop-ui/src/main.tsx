import React, { useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/tauri";
import "./styles.css";

type DeviceMode = "Normal" | "Recovery" | "Dfu" | "Bootloader" | "Fastboot" | "Adb" | "MassStorage" | "Unknown";

interface DeviceInfo {
  bus_number: number;
  address: number;
  vendor_id: number;
  product_id: number;
  vendor_name?: string | null;
  manufacturer?: string | null;
  product_name?: string | null;
  serial_number?: string | null;
  platform: string;
  transport: string;
  mode: DeviceMode;
  recommended_workflow?: string;
}

const demoDevices: DeviceInfo[] = [
  {
    bus_number: 1,
    address: 4,
    vendor_id: 0x05ac,
    product_id: 0x12a8,
    vendor_name: "Apple",
    manufacturer: "Apple Inc.",
    product_name: "iPhone",
    serial_number: "DEMO-DEVICE",
    platform: "Apple",
    transport: "Usb3",
    mode: "Normal",
    recommended_workflow: "StandardInspection",
  },
];

const hex = (value: number) => value.toString(16).padStart(4, "0").toUpperCase();

function isDesktopRuntime() {
  return "__TAURI__" in window;
}

function App() {
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [selected, setSelected] = useState<number | null>(null);
  const [scanning, setScanning] = useState(false);
  const [message, setMessage] = useState("Ready for a read-only USB scan.");

  const active = selected === null ? undefined : devices[selected];
  const recoveryCount = useMemo(
    () => devices.filter((device) => device.mode !== "Normal" && device.mode !== "Unknown").length,
    [devices],
  );

  async function scan() {
    setScanning(true);
    setMessage("Inspecting USB descriptors…");
    try {
      const result = isDesktopRuntime()
        ? await invoke<DeviceInfo[]>("scan_connected_devices")
        : demoDevices;
      setDevices(result);
      setSelected(result.length > 0 ? 0 : null);
      setMessage(
        isDesktopRuntime()
          ? `${result.length} connected device${result.length === 1 ? "" : "s"} detected.`
          : "Preview mode: showing a safe sample device.",
      );
    } catch (error) {
      setDevices([]);
      setSelected(null);
      setMessage(`Scan unavailable: ${String(error)}`);
    } finally {
      setScanning(false);
    }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand-mark" aria-hidden="true">P</div>
        <div className="brand-copy">
          <span>Phoenix Key</span>
          <small>powered by BootForge</small>
        </div>
        <nav aria-label="Primary">
          <button className="nav-item active"><span>⌁</span> Device Forge</button>
          <button className="nav-item" disabled><span>◇</span> Media Builder</button>
          <button className="nav-item" disabled><span>↻</span> Recovery Center</button>
          <button className="nav-item" disabled><span>▦</span> Session History</button>
        </nav>
        <div className="safety-card">
          <strong>Read-only foundation</strong>
          <p>This MVP identifies hardware and recommends a safe next route. It does not modify devices.</p>
        </div>
        <footer>Reignite · Rebuild · Reboot</footer>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">DEVICE FORGE</p>
            <h1>Know what is connected before you act.</h1>
          </div>
          <div className="runtime-pill">
            <i className={isDesktopRuntime() ? "online" : "preview"} />
            {isDesktopRuntime() ? "Desktop engine" : "Browser preview"}
          </div>
        </header>

        <div className="hero-panel">
          <div>
            <span className="status-label">FORGE STATUS</span>
            <h2>{scanning ? "Reading the signal…" : "Phoenix Key is standing by."}</h2>
            <p>{message}</p>
          </div>
          <button className="scan-button" onClick={scan} disabled={scanning}>
            {scanning ? "Scanning…" : "Scan USB Devices"}
          </button>
        </div>

        <div className="metric-grid">
          <Metric label="Connected" value={devices.length.toString()} detail="visible USB devices" />
          <Metric label="Special modes" value={recoveryCount.toString()} detail="recovery or service states" />
          <Metric label="Safety mode" value="READ" detail="descriptor inspection only" accent />
        </div>

        <div className="content-grid">
          <section className="device-list panel">
            <div className="panel-heading">
              <div><p className="eyebrow">CONNECTED HARDWARE</p><h3>Device inventory</h3></div>
              <span>{devices.length}</span>
            </div>
            {devices.length === 0 ? (
              <div className="empty-state">
                <div className="port-icon">⌁</div>
                <h4>No scan results yet</h4>
                <p>Connect a device, then run a descriptor scan.</p>
              </div>
            ) : (
              <div className="device-rows">
                {devices.map((device, index) => (
                  <button
                    className={`device-row ${selected === index ? "selected" : ""}`}
                    key={`${device.bus_number}-${device.address}-${device.vendor_id}-${device.product_id}`}
                    onClick={() => setSelected(index)}
                  >
                    <span className="device-orb">{device.platform === "Apple" ? "A" : "U"}</span>
                    <span><strong>{device.product_name || "USB Device"}</strong><small>{hex(device.vendor_id)}:{hex(device.product_id)}</small></span>
                    <b>{device.mode}</b>
                  </button>
                ))}
              </div>
            )}
          </section>

          <section className="details panel">
            <div className="panel-heading">
              <div><p className="eyebrow">SIGNAL REPORT</p><h3>Device details</h3></div>
            </div>
            {active ? (
              <div className="detail-body">
                <div className="device-title">
                  <span className="device-orb large">{active.platform === "Apple" ? "A" : "U"}</span>
                  <div><h4>{active.product_name || "USB Device"}</h4><p>{active.manufacturer || active.vendor_name || "Unknown manufacturer"}</p></div>
                </div>
                <dl>
                  <Detail label="Hardware ID" value={`${hex(active.vendor_id)}:${hex(active.product_id)}`} />
                  <Detail label="Mode" value={active.mode} />
                  <Detail label="Platform" value={active.platform} />
                  <Detail label="Transport" value={active.transport} />
                  <Detail label="Bus / Address" value={`${active.bus_number} / ${active.address}`} />
                  <Detail label="Serial" value={active.serial_number || "Not exposed"} />
                </dl>
                <div className="recommendation">
                  <span>RECOMMENDED ROUTE</span>
                  <strong>{active.recommended_workflow || "Standard inspection"}</strong>
                  <p>Review the detected identity and mode before continuing into a guided workflow.</p>
                </div>
              </div>
            ) : (
              <div className="empty-state compact"><p>Select a detected device to open its signal report.</p></div>
            )}
          </section>
        </div>
      </section>
    </main>
  );
}

function Metric({ label, value, detail, accent = false }: { label: string; value: string; detail: string; accent?: boolean }) {
  return <article className={`metric ${accent ? "accent" : ""}`}><span>{label}</span><strong>{value}</strong><p>{detail}</p></article>;
}

function Detail({ label, value }: { label: string; value: string }) {
  return <div><dt>{label}</dt><dd>{value}</dd></div>;
}

createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>);
